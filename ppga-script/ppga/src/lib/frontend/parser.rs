//! Implements the PPGA parser using the recursive descent algorithm and an EBNF-like grammar .
use logos::Lexer as RawLexer;

use super::{super::errors::*, ast::*, lexer::*};
use crate::{
    codegen::snippets::{DEFAULT_OP_NAME, ERR_CALLBACK_NAME, ERR_HANDLER_NAME},
    PPGAConfig,
};

/// The lexer type expected by the parser.
pub type Lexer<'a> = RawLexer<'a, TokenKind<'a>>;
type ExprRes<'a> = Result<Expr<'a>, ParseError>;
type StmtRes<'a> = Result<Stmt<'a>, ParseError>;

/// A recursive descent PPGA parser.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    eof: Token<'a>,
    line: usize,
    ex: ErrCtx<'a>,
    /// The comments encountered since the last statement
    comments: Vec<Comment<'a>>,
    config: PPGAConfig,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from the given lexer.
    pub fn new(lexer: Lexer<'a>) -> Self {
        let ex = ErrCtx::new(lexer.source());

        let end = if ex.source.is_empty() {
            0
        } else {
            ex.source.len() - 1
        };
        let eof = Token::new("", ex.lines.len() - 1, end..end, TokenKind::EOF);

        let mut p = Self {
            lexer,
            previous: eof.clone(),
            current: eof.clone(),
            eof,
            line: 0,
            ex,
            comments: Vec::new(),
            config: PPGAConfig::default(),
        };
        // Scan the first two tokens
        p.advance();
        p
    }

    /// Creates a new parser from the given config and lexer.
    pub fn with_config(config: PPGAConfig, lexer: Lexer<'a>) -> Self {
        Self {
            config,
            ..Self::new(lexer)
        }
    }

    /// Consumes the parser, returning either the parsed AST or the list of encountered errors.
    pub fn parse(mut self) -> Result<AST<'a>, ErrCtx<'a>> {
        let mut statements = vec![];

        while !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.ex.record(e);
                    self.synchronize()
                }
            }
        }

        if self.ex.had_error() {
            Err(self.ex)
        } else {
            Ok(AST {
                stmts: statements,
                comments: self.comments,
                config: self.config,
            })
        }
    }

    fn statement(&mut self) -> Result<Stmt<'a>, ParseError> {
        if self.match_any(&[TokenKind::Let, TokenKind::Global]) {
            self.var_declaration()
        } else if self.r#match(TokenKind::LeftBrace) {
            self.block(true)
        } else if self.r#match(TokenKind::Fn) {
            let name = self.consume_identifier("Expected a function name after the keyword.")?;
            let mut r#fn = self.lambda()?;
            r#fn.name = Some(name.lexeme);
            return Ok(stmt!(
                self,
                StmtKind::FuncDecl(VarKind::Local, Ptr::new(r#fn))
            ));
        } else if self.r#match(TokenKind::If) {
            self.if_statement()
        } else if self.r#match(TokenKind::For) {
            self.for_statement(false)
        } else if self.r#match(TokenKind::ForI) {
            self.for_statement(true)
        } else if self.r#match(TokenKind::While) {
            self.while_statement()
        } else if self.r#match(TokenKind::Return) {
            let kind = StmtKind::Return(self.arguments(TokenKind::Semicolon)?);
            self.consume_semicolon("Expected a `;` after the return")?;
            Ok(stmt!(self, kind))
        } else if self.r#match(TokenKind::Break) {
            self.consume_semicolon("Expected a `;` after the break")?;
            Ok(stmt!(self, StmtKind::Break))
        // the number doesn't matter
        } else if self.r#match(TokenKind::EOLSeq(0)) {
            let n = match self.previous().kind {
                TokenKind::EOLSeq(n) => n,
                _ => unreachable!("{:#?}", self.previous()),
            };
            Ok(stmt!(self, StmtKind::NewLine(if n < 2 { 0 } else { 1 })))
        } else {
            let stmt = self.assignment()?;
            match &stmt.kind {
                StmtKind::ExprStmt(box Expr {
                    kind: ExprKind::LuaBlock(_),
                    ..
                }) => (),
                _ => {
                    self.consume_semicolon("Expected a `;` after the expression")?;
                }
            }
            Ok(stmt)
        }
    }

    fn var_declaration(&mut self) -> StmtRes<'a> {
        let kind = match self.previous().kind {
            TokenKind::Let => VarKind::Local,
            TokenKind::Global => VarKind::Global,
            _ => unreachable!("{:?}", self.previous()),
        };
        if kind == VarKind::Global && self.r#match(TokenKind::Fn) {
            let name = self.consume_identifier("Expected a function name after the keyword.")?;
            let mut r#fn = self.lambda()?;
            r#fn.name = Some(name.lexeme);
            return Ok(stmt!(self, StmtKind::FuncDecl(kind, Ptr::new(r#fn))));
        }
        let mut names =
            vec![self.consume_identifier("Expected a variable name after the keyword.")?];
        while self.r#match(TokenKind::Comma) {
            names.push(self.consume_identifier("Expected a variable name after the comma.")?);
        }

        let mut initializer = if self.r#match(TokenKind::Equal) {
            Some(Ptr::new(self.expression()?))
        } else {
            if kind == VarKind::Global {
                let ident = names.first().unwrap();
                self.ex.record(ParseError::new(
                    ident.line,
                    ident.span.clone(),
                    "Global variables must be assigned a value",
                ));
            }
            if self.r#match(TokenKind::Query) {
                let token = self.previous().clone();
                self.ex.record(ParseError::new(
                    token.line,
                    token.span,
                    "Cannot use `?` without an initializer",
                ));
            }

            None
        };

        let perform_error_expansion = self.consume(TokenKind::Query, "").ok();
        self.consume_semicolon("Expected a `;` after the variable declaration")?;

        if perform_error_expansion.is_some() && names.len() != 1 {
            let q = perform_error_expansion.unwrap();
            return Err(ParseError::new(
                q.line,
                q.span,
                "Cannot use `?` with more than one variable name.",
            ));
        }

        let stmt = if let Some(query) = perform_error_expansion {
            if !Parser::has_err_block(&*initializer.as_ref().unwrap()) {
                initializer = Some(Ptr::new(
                    make_call!(alloc => owned_var!(ERR_HANDLER_NAME), owned_var!(ERR_CALLBACK_NAME), *initializer.unwrap()),
                ));
            }
            // Generate a let statement that initializes the variable to nil
            let target_var = names.first().unwrap().lexeme;
            let decl = stmt!(
                self,
                StmtKind::VarDecl(
                    kind,
                    vec![VarName::Borrowed(target_var)],
                    Some(Ptr::new(make_literal!("nil")))
                )
            );

            let ok_name = format!("_ok_L{}S{}", query.line, query.span.start);
            let err_name = format!("_err_L{}S{}", query.line, query.span.start);
            let err_var = owned_var!(err_name.clone());
            let ok_var = owned_var!(ok_name.clone());
            let block = make_block!(
                true,
                // Generate the tuple destruction statement:
                // let ok, err = <initializer>;
                stmt!(
                    self,
                    StmtKind::VarDecl(
                        kind,
                        vec![VarName::Owned(ok_name), VarName::Owned(err_name)],
                        initializer
                    )
                ),
                // Check if the error is `nil` and return the error if it is
                stmt!(
                    self,
                    StmtKind::If(
                        make_binary!(alloc => err_var.clone(), "!=", make_literal!("nil")),
                        Ptr::new(make_block!(false, make_return!(err_var))),
                        None
                    )
                ),
                // Assign the ok to the variable
                stmt!(
                    self,
                    StmtKind::Assignment(vec![make_var!(target_var)], "=", Ptr::new(ok_var))
                )
            );

            stmt!(self, StmtKind::StmtSequence(vec![decl, block]))
        } else {
            stmt!(
                self,
                StmtKind::VarDecl(
                    kind,
                    // XXX: Should we lint the usage of variadics in var declarations?
                    names
                        .into_iter()
                        .map(|n| VarName::Borrowed(n.lexeme))
                        .collect(),
                    initializer
                )
            )
        };
        Ok(stmt)
    }

    fn has_err_block(expr: &Expr<'a>) -> bool {
        let mut node = Some(expr);

        while let Some(expr) = node {
            match &expr.kind {
                ExprKind::Call(callee, _) => match &callee.kind {
                    ExprKind::GeneratedVariable(v) if v == ERR_HANDLER_NAME => {
                        return true;
                    }
                    _ => break,
                },
                ExprKind::Grouping(ref expr) => {
                    node = Some(expr);
                }
                _ => break,
            }
        }

        false
    }

    fn block(&mut self, is_standalone: bool) -> StmtRes<'a> {
        let mut statements = vec![];
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.ex.record(e);
                    self.synchronize();
                }
            }
        }
        self.consume(TokenKind::RightBrace, "Expected a `}` after the block.")?;
        Ok(stmt!(self, StmtKind::Block(statements, is_standalone)))
    }

    fn if_statement(&mut self) -> StmtRes<'a> {
        let condition = self.expression()?;
        self.consume(
            TokenKind::LeftBrace,
            "Expected a `{` after the if condition.",
        )?;
        let then = self.block(false)?;
        let r#else = if self.r#match(TokenKind::Else) {
            if self.r#match(TokenKind::If) {
                Some(Ptr::new(self.if_statement()?))
            } else {
                self.consume(
                    TokenKind::LeftBrace,
                    "Expected a `{` or an `if` after the `else`.",
                )?;
                Some(Ptr::new(self.block(false)?))
            }
        } else {
            None
        };

        Ok(stmt!(
            self,
            StmtKind::If(Ptr::new(condition), Ptr::new(then), r#else)
        ))
    }

    fn for_statement(&mut self, is_fori: bool) -> StmtRes<'a> {
        let line = self.line;
        let vars = self.parameters()?;
        if vars.is_empty() {
            return Err(ParseError::new(
                self.line,
                self.previous().span.clone(),
                "Expected an identifier after the loop keyword",
            ));
        }
        self.consume(TokenKind::In, "Expected an `in` after the loop variables")?;

        let condition = if self.r#match(TokenKind::Range) {
            let range = ForCondition::Range(self.range()?);
            if is_fori {
                return Err(ParseError::new(
                    line,
                    self.previous().span.clone(),
                    "A range cannot be used with a fori loop.",
                ));
            }
            range
        } else {
            let args = self.arguments(TokenKind::LeftBrace)?;
            ForCondition::Exprs(args)
        };

        self.consume(
            TokenKind::LeftBrace,
            "Expected a `{` after the loop condition",
        )?;
        let body = Ptr::new(self.block(false)?);
        Ok(stmt!(
            self,
            StmtKind::For(ForLoop {
                is_fori,
                vars,
                condition,
                body
            })
        ))
    }

    fn while_statement(&mut self) -> StmtRes<'a> {
        let condition = self.expression()?;
        self.consume(
            TokenKind::LeftBrace,
            "Expected a `{` after the loop condition",
        )?;
        let block = self.block(false)?;
        Ok(stmt!(
            self,
            StmtKind::While(Ptr::new(condition), Ptr::new(block))
        ))
    }

    fn assignment(&mut self) -> StmtRes<'a> {
        let mut exprs = vec![self.expression()?];

        while self.r#match(TokenKind::Comma) {
            exprs.push(self.expression()?);
        }

        if self.match_any(&[
            TokenKind::Equal,
            TokenKind::PlusEqual,
            TokenKind::MinusEqual,
            TokenKind::StarEqual,
            TokenKind::SlashEqual,
            TokenKind::PowEqual,
        ]) {
            let span = self.previous().span.clone();
            let line = self.previous().line;
            let operator = self.previous().lexeme;
            for expr in &exprs {
                match expr.kind {
                    ExprKind::Variable(_)
                    | ExprKind::Get(_, _, false)
                    | ExprKind::GetItem(_, _) => (),
                    _ => return Err(ParseError::new(line, span, "Invalid assignment target")),
                }
            }
            return Ok(stmt!(
                self,
                StmtKind::Assignment(exprs, operator, Ptr::new(self.expression()?))
            ));
        }

        if exprs.len() > 1 {
            return Err(ParseError::new(
                self.line,
                self.previous().span.clone(),
                "Comma is allowed only in let, assignment, and return statements.",
            ));
        }

        let expr = exprs.drain(0..).next().unwrap();
        Ok(stmt!(self, StmtKind::ExprStmt(Ptr::new(expr))))
    }

    fn range(&mut self) -> Result<Ptr<Range>, ParseError> {
        self.consume(TokenKind::LeftParen, "Expected a `(` after `range`")?;
        let mut step = 1 as Number;
        let mut start = 0 as Number;
        let mut end = Self::as_number(
            self.consume(TokenKind::Number, "Expected a range stop value")?
                .lexeme,
        );
        if self.r#match(TokenKind::Comma) {
            start = end;
            end = Self::as_number(
                self.consume(TokenKind::Number, "Expected a range stop value")?
                    .lexeme,
            );
        };
        if self.r#match(TokenKind::Comma) {
            step = Self::as_number(
                self.consume(TokenKind::Number, "Expected a range step value")?
                    .lexeme,
            );
        }
        self.consume(TokenKind::RightParen, "Expected a `)` after the arguments")?;
        Ok(Ptr::new(Range { start, end, step }))
    }

    fn expression(&mut self) -> ExprRes<'a> {
        if self.r#match(TokenKind::Fn) {
            Ok(expr!(self, ExprKind::Lambda(Ptr::new(self.lambda()?))))
        } else {
            self.default_op()
        }
    }

    fn lambda(&mut self) -> Result<Function<'a>, ParseError> {
        self.consume(
            TokenKind::LeftParen,
            "Expected a `(` before the parameter list.",
        )?;

        let params = if self.check(&TokenKind::RightParen) {
            vec![]
        } else {
            self.parameters()?
        };

        self.consume(
            TokenKind::RightParen,
            "Expected a `)` after the parameter list.",
        )?;

        let body = if self.r#match(TokenKind::FatArrow) {
            stmt!(
                self,
                StmtKind::Block(
                    vec![stmt!(self, StmtKind::Return(vec![self.expression()?]))],
                    false
                )
            )
        } else {
            self.consume(
                TokenKind::LeftBrace,
                "Expected a `{` or a `=>` after the parameter list.",
            )?;
            self.block(false)?
        };

        Ok(Function {
            name: None,
            params,
            body,
        })
    }

    fn default_op(&mut self) -> ExprRes<'a> {
        let mut expr = self.logic_or()?;

        if self.r#match(TokenKind::DoubleQuery) {
            let right = self.logic_or()?;
            expr = expr!(
                self,
                ExprKind::Call(
                    Ptr::new(expr!(sf, ExprKind::Variable(DEFAULT_OP_NAME))),
                    vec![expr, right]
                )
            );
        }

        Ok(expr)
    }

    fn logic_or(&mut self) -> ExprRes<'a> {
        let mut expr = self.logic_and()?;

        while self.r#match(TokenKind::Or) && !self.is_at_end() {
            let operator = self.previous().clone().lexeme;
            let right = self.logic_and()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> ExprRes<'a> {
        let mut expr = self.equality()?;

        while self.r#match(TokenKind::And) && !self.is_at_end() {
            let operator = self.previous().clone().lexeme;
            let right = self.equality()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ExprRes<'a> {
        let mut expr = self.comparison()?;

        while self.match_any(&[TokenKind::Ne, TokenKind::Eq]) {
            let operator = self.previous().lexeme;
            let right = self.comparison()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ExprRes<'a> {
        let mut expr = self.addition()?;

        while self.match_any(&[TokenKind::Lt, TokenKind::Le, TokenKind::Gt, TokenKind::Ge]) {
            let operator = self.previous().lexeme;
            let right = self.addition()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn addition(&mut self) -> ExprRes<'a> {
        let mut expr = self.multiplication()?;

        while self.match_any(&[TokenKind::Plus, TokenKind::Minus, TokenKind::DoubleDot]) {
            let operator = self.previous().lexeme;
            let right = self.multiplication()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> ExprRes<'a> {
        let mut expr = self.exponentiation()?;

        while self.match_any(&[
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::BackSlash,
            TokenKind::Percent,
        ]) {
            let operator = match self.previous().lexeme {
                "\\" => "//",
                lex => lex,
            };
            let right = self.exponentiation()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn exponentiation(&mut self) -> ExprRes<'a> {
        let mut expr = self.unary()?;

        while self.r#match(TokenKind::Pow) {
            let operator = self.previous().lexeme;
            let right = self.exponentiation()?;
            expr = expr!(
                self,
                ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right))
            );
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ExprRes<'a> {
        let expr = if self.match_any(&[TokenKind::Minus, TokenKind::Not, TokenKind::Ellipsis]) {
            let operator = self.previous().lexeme;
            let value = self.unary()?;
            expr!(self, ExprKind::Unary(operator, Ptr::new(value)))
        } else {
            self.call()?
        };

        Ok(expr)
    }

    fn call(&mut self) -> ExprRes<'a> {
        let mut expr = if self.r#match(TokenKind::Len) {
            self.finish_len()?
        } else {
            self.primary()?
        };

        while self.match_any(&[
            TokenKind::LeftParen,
            TokenKind::Dot,
            TokenKind::Colon,
            TokenKind::LeftBracket,
        ]) {
            let kind = &self.previous().kind;
            expr = match kind {
                TokenKind::LeftParen => self.finish_call(expr)?,
                TokenKind::LeftBracket => self.finish_item(expr)?,
                kind => {
                    let is_static = kind == &TokenKind::Colon;
                    self.finish_attr(expr, is_static)?
                }
            };

            while self.check(&TokenKind::Identifier) {
                if self.peek().lexeme != "err" {
                    break;
                }
                let _err = self.advance();
                self.consume(
                    TokenKind::LeftBrace,
                    "Expected a `{` after the `err` in an err block.",
                )?;
                let body = self.block(false)?;
                expr = Parser::make_err_block(body, expr);
            }
        }

        Ok(expr)
    }

    fn make_err_block(callback_body: Stmt<'a>, arg: Expr<'a>) -> Expr<'a> {
        let callback = expr!(
            self,
            ExprKind::Lambda(Ptr::new(Function {
                name: None,
                params: vec![owned_var!("err")],
                body: callback_body,
            }))
        );
        make_call!(alloc => owned_var!(ERR_HANDLER_NAME), callback, arg)
    }

    fn primary(&mut self) -> ExprRes<'a> {
        log::trace!("primary {:?} {:?}", self.previous(), self.peek());
        let token = self.advance().clone();
        log::trace!("primary w/ token {:?} {:?}", self.previous(), self.peek());
        let expr = match token.kind {
            TokenKind::Nil | TokenKind::True | TokenKind::False | TokenKind::Number => {
                expr!(self, ExprKind::Literal(token.lexeme, false))
            }
            TokenKind::StringLiteral => expr!(
                self,
                ExprKind::Literal(
                    if !token.lexeme.is_empty() {
                        &token.lexeme[1..token.lexeme.len() - 1]
                    } else {
                        token.lexeme
                    },
                    true
                )
            ),
            TokenKind::Identifier => expr!(self, ExprKind::Variable(token.lexeme)),
            TokenKind::Variadics => expr!(self, ExprKind::Variable("...")),
            TokenKind::EOLSeq(n) => {
                self.bump_line(n);
                expr!(self, ExprKind::NewLine)
            }
            TokenKind::LeftParen => {
                let kind = ExprKind::Grouping(Ptr::new(self.expression()?));
                self.consume(
                    TokenKind::RightParen,
                    "Expected a `)` after the expression.",
                )?;
                expr!(self, kind)
            }
            TokenKind::LeftBracket => {
                let initializers = if !self.check(&TokenKind::RightBracket) {
                    self.arguments(TokenKind::RightBracket)?
                } else {
                    vec![]
                };

                self.consume(
                    TokenKind::RightBracket,
                    "Expected a `]` at the end of an array literal.",
                )?;

                expr!(self, ExprKind::ArrayLiteral(initializers))
            }
            TokenKind::LeftBrace => {
                let pairs = if !self.check(&TokenKind::RightBrace) {
                    self.pairs()?
                } else {
                    vec![]
                };

                self.consume(
                    TokenKind::RightBrace,
                    "Expected a `}` at the end of a dict literal.",
                )?;

                expr!(self, ExprKind::DictLiteral(pairs))
            }
            TokenKind::InterpolatedString(frags) => self.finish_interpolated_string(frags)?,
            TokenKind::Lua(s) => {
                self.bump_line(s.matches("\n").count());
                expr!(self, ExprKind::LuaBlock(s))
            }
            TokenKind::EOF => {
                return Err(ParseError::new(
                    self.eof.line,
                    self.eof.span.clone(),
                    format!(
                        "Reached the end of the script, last seen token was {:?}",
                        self.previous().kind
                    ),
                ));
            }
            _ => {
                return Err(ParseError::new(
                    token.line,
                    token.span,
                    format!("Unexpected symbol `TokenKind::{:?}`", token.kind),
                ));
            }
        };
        Ok(expr)
    }

    fn finish_interpolated_string(&mut self, frags: Vec<Frag<'a>>) -> ExprRes<'a> {
        let mut exprs = vec![];

        let mut frags = frags.into_iter();
        loop {
            if let Some(frag) = frags.next() {
                match frag {
                    Frag::Str(s) => {
                        // Skip empty strings
                        if s.lexeme == "" {
                            continue;
                        }
                        exprs.push(expr!(self, ExprKind::Literal(s.lexeme, true)));
                    }
                    Frag::Sublexer(lex) => {
                        let expr = Parser::new(lex).expression().map_err(|mut e| {
                            e.span.line = self.previous().line;
                            e
                        })?;
                        exprs.push(expr);
                    }
                }
            } else {
                break;
            }
        }
        Ok(expr!(self, ExprKind::FString(exprs)))
    }

    fn finish_call(&mut self, callee: Expr<'a>) -> ExprRes<'a> {
        let args = if !self.check(&TokenKind::RightParen) {
            let args = match self.arguments(TokenKind::RightParen) {
                Ok(args) => args,
                Err(e) => {
                    let _ = self
                        .consume(
                            TokenKind::RightParen,
                            "Expected a ')' after the arguments list.",
                        )
                        .map_err(|e| self.ex.record(e));
                    return Err(e);
                }
            };
            args
        } else {
            vec![]
        };
        let _ = self
            .consume(
                TokenKind::RightParen,
                "Expected a ')' after the arguments list.",
            )
            .map_err(|e| self.ex.record(e));
        Ok(expr!(self, ExprKind::Call(Ptr::new(callee), args)))
    }

    fn finish_len(&mut self) -> ExprRes<'a> {
        self.consume(
            TokenKind::LeftParen,
            "Expected a `(` before the len argument",
        )?;
        let expr = self.expression()?;
        self.consume(
            TokenKind::RightParen,
            "Expected a `)` after the len argument",
        )?;
        Ok(expr!(self, ExprKind::Len(Ptr::new(expr))))
    }

    fn finish_item(&mut self, obj: Expr<'a>) -> ExprRes<'a> {
        let key = self.expression()?;
        self.consume(TokenKind::RightBracket, "Expected a `]` after")?;
        Ok(expr!(self, ExprKind::GetItem(Ptr::new(obj), Ptr::new(key))))
    }

    fn finish_attr(&mut self, obj: Expr<'a>, is_static: bool) -> ExprRes<'a> {
        let attr = self.consume_identifier("Expected an attribute name after the dot.")?;
        Ok(expr!(
            self,
            ExprKind::Get(Ptr::new(obj), attr.lexeme, is_static)
        ))
    }

    fn parameters(&mut self) -> Result<Vec<Expr<'a>>, ParseError> {
        let mut params = vec![];

        if !self.check(&TokenKind::RightParen) {
            let expr = self.consume_identifier("Expected an identifier.")?;
            // TODO: disable optional idents when parsing for loop variables
            let optional = self.r#match(TokenKind::Query);
            params.push(expr!(self, ExprKind::Param(&expr.lexeme, optional)));
        }

        while self.r#match(TokenKind::Comma) {
            if self.check(&TokenKind::RightParen) {
                break;
            }
            let expr = self.consume_identifier("Expected an identifier.")?;
            let optional = self.r#match(TokenKind::Query);
            params.push(expr!(self, ExprKind::Param(&expr.lexeme, optional)));
        }

        Ok(params)
    }

    fn arguments(&mut self, stop: TokenKind<'a>) -> Result<Vec<Expr<'a>>, ParseError> {
        let mut args = vec![];

        if !self.check(&stop) {
            let expr = self.expression()?;
            args.push(expr);
        }

        while self.r#match(TokenKind::Comma) && !self.is_at_end() {
            if self.check(&stop) {
                break;
            }
            args.push(self.expression()?);
        }

        Ok(args)
    }

    fn pairs(&mut self) -> Result<Vec<(Expr<'a>, Expr<'a>)>, ParseError> {
        let mut pairs = vec![];

        if !self.check(&TokenKind::RightBrace) {
            let key = self.expression()?;
            self.consume(TokenKind::Equal, "Expected an `=` after a key.")?;
            let value = self.expression()?;
            pairs.push((key, value));
        }

        while self.r#match(TokenKind::Comma) && !self.is_at_end() {
            if self.check(&TokenKind::RightBrace) {
                break;
            }
            let key = self.expression()?;
            self.consume(TokenKind::Equal, "Expected an `=` after a key.")?;
            let value = self.expression()?;
            pairs.push((key, value));
        }

        Ok(pairs)
    }

    #[inline]
    fn consume_identifier<S: Into<String>>(&mut self, msg: S) -> Result<Token<'a>, ParseError> {
        if self.check(&TokenKind::Variadics) {
            self.consume(TokenKind::Variadics, msg)
        } else {
            self.consume(TokenKind::Identifier, msg)
        }
    }

    fn consume_semicolon<S: Into<String>>(&mut self, msg: S) -> Result<Token<'a>, ParseError> {
        if self.check(&TokenKind::Semicolon) {
            // allow multiple semicolons
            while self.r#match(TokenKind::Semicolon) {}

            Ok(self.previous().clone())
        } else {
            let line = self.current.line;
            let span = self.current.span.clone();
            Err(ParseError::new(line, span, msg))
        }
    }

    fn consume<S: Into<String>>(
        &mut self,
        kind: TokenKind<'a>,
        msg: S,
    ) -> Result<Token<'a>, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(
                self.peek().line,
                self.peek().span.clone(),
                msg,
            ))
        }
    }

    fn r#match(&mut self, kind: TokenKind<'a>) -> bool {
        if self.check(&kind) {
            self.previous = self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, kinds: &[TokenKind<'a>]) -> bool {
        for kind in kinds {
            if self.check(&kind) {
                self.previous = self.advance();
                return true;
            }
        }

        false
    }

    fn previous(&self) -> &Token<'a> {
        &self.previous
    }

    #[inline(always)]
    fn peek(&self) -> &Token<'a> {
        &self.current
    }

    #[inline]
    fn check(&mut self, kind: &TokenKind<'a>) -> bool {
        if [
            TokenKind::EOL,
            TokenKind::Comment,
            TokenKind::MultilineComment,
        ]
        .contains(&self.current.kind)
        {
            self.advance();
        }
        std::mem::discriminant(kind) == std::mem::discriminant(&self.current.kind)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            if [
                TokenKind::Fn,
                TokenKind::Let,
                TokenKind::Global,
                TokenKind::For,
                TokenKind::ForI,
                TokenKind::If,
                TokenKind::While,
                TokenKind::Return,
                TokenKind::LeftBrace,
            ]
            .contains(&self.peek().kind)
            {
                return;
            }

            self.advance();
        }
    }

    fn advance(&mut self) -> Token<'a> {
        let mut new_comments = vec![];
        let mut token = self.advance_and_skip_newlines();

        while [TokenKind::Comment, TokenKind::MultilineComment].contains(&token.kind) {
            new_comments.push(match token.kind {
                TokenKind::Comment => Comment::Regular(&token.lexeme[2..]),
                TokenKind::MultilineComment => {
                    self.bump_line(
                        token.lexeme[2..token.lexeme.len() - 1]
                            .matches("\n")
                            .count(),
                    );
                    Comment::Multiline(&token.lexeme[2..token.lexeme.len() - 1])
                }
                _ => unreachable!(),
            });
            token = self.advance_and_skip_newlines();
        }

        if self.config.emit_comments {
            self.comments.extend(new_comments);
        }

        self.r#match(TokenKind::EOL);

        token
    }

    fn advance_and_skip_newlines(&mut self) -> Token<'a> {
        match self.current.kind {
            TokenKind::EOL => {
                // QQQ: do we even need this while loop? Only single EOLS can appear by themselves
                //      so this loop seems to be redundant.
                while self.current.kind == TokenKind::EOL {
                    self.bump_line(1);
                    self.actual_advance();
                }
                self.previous.clone()
            }
            TokenKind::EOLSeq(n) => {
                self.bump_line(n);
                self.actual_advance().clone()
            }
            _ => self.actual_advance().clone(),
        }
    }

    fn actual_advance(&mut self) -> &Token<'a> {
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current = self
            .lexer
            .next()
            .map(|kind| Token::new(self.lexer.slice(), self.line, self.lexer.span(), kind))
            .unwrap_or_else(|| self.eof.clone());
        &self.previous
    }

    #[inline(always)]
    fn as_number(lexeme: &str) -> Number {
        match lexeme.parse() {
            Ok(value) => value,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn bump_line(&mut self, n: usize) {
        self.line += n;
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.current.kind == TokenKind::EOF
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::lexer;

    #[test]
    fn test_comma_is_allowed_only_in_return_let_and_assignment() {
        let sources = vec![
            "let a, b = 3;",
            "a, b = @;",
            "return a, b;",
            "a, b;",
            "let d = a, b + 3;",
        ];
        let expected = vec![
            Ok(()),
            Ok(()),
            Ok(()),
            Err("Comma is allowed only in let, assignment, and return statements."),
            Err("Expected a `;` after the variable declaration"),
        ];

        for (i, (s, e)) in sources.iter().zip(expected.iter()).enumerate() {
            let parser = Parser::new(lexer(s));
            let res = parser.parse();
            match (e, &res) {
                (Ok(_), Ok(_)) => (),
                (e @ Ok(_), r @ Err(_)) | (e @ Err(_), r @ Ok(_)) => panic!(
                    "[Test #{}] Succeeded or failed without a reason: {:#?}\nwhile expected: {:#?}",
                    i, r, e
                ),
                (Err(e), Err(r)) => {
                    r.report_all();
                    assert_eq!(e, &r.errors[0].message);
                }
            }
        }
    }
}
