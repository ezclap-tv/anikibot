use crate::BoxedError;
use logos::{Lexer as RawLexer, Span};

pub type Lexer<'a> = RawLexer<'a, TokenKind>;
type ExprRes<'a> = Result<Expr<'a>, ParseError>;
type StmtRes<'a> = Result<Stmt<'a>, ParseError>;

use super::{
    ast::{Expr, ExprKind, Ptr, Range, Stmt, StmtKind, VarKind, AST},
    lexer::*,
};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn new<S: Into<String>>(span: Span, message: S) -> ParseError {
        Self {
            span,
            message: message.into(),
        }
    }
}

pub struct Parser<'a> {
    source: &'a str,
    lexer: Lexer<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    eof: Token<'static>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let source = lexer.source();
        let end = source.len() - 1;
        let eof = Token::new("", end..end, TokenKind::EOF);

        Self {
            lexer,
            source,
            previous: eof,
            current: eof,
            eof,
        }
    }

    pub fn parse(self) -> Result<AST<'a>, ParseError> {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(AST { stmts: statements })
    }

    pub fn statement(&mut self) -> Result<Stmt<'a>, ParseError> {
        match self.peek().kind {
            TokenKind::Let | TokenKind::Global => self.var_declaration(),
            _ => unreachable!(),
            // Token::Fn => fn_declaration(token, true, &mut lex),
            // Token::LeftBrace => block(token, &mut lex),
            // Token::If => if_statement(token, &mut lex),
            // Token::For => for_statement(token, &mut lex),
            // Token::ForI => fori_statement(token, &mut lex),
            // Token::While => while_statement(token, &mut lex),
            // Token::Return => return_statement(token, &mut lex),
            // Token::Break => break_statement(token, &mut lex),
            // Token::Continue => continue_statement(token, &mut lex),
            // _ => expression_statement(token, &mut lex),
        }
    }

    fn var_declaration(&mut self) -> StmtRes<'a> {
        let kind = match self.advance().kind {
            TokenKind::Let => VarKind::Local,
            TokenKind::Global => VarKind::Global,
            _ => unreachable!(),
        };
        let ident = self.consume_identifier("Expected a variable name after the keyword.")?;
        let initializer = if self.r#match(TokenKind::Equal) {
            Some(Ptr::new(self.expression()?))
        } else {
            None
        };
        Ok(Stmt {
            kind: StmtKind::VarDecl(kind, ident.lexeme, initializer),
        })
    }

    fn expression(&mut self) -> ExprRes<'a> {
        if self.r#match(TokenKind::Fn) {
            self.lambda()
        } else {
            self.logic_or()
        }
    }

    fn logic_or(&mut self) -> ExprRes<'a> {
        let mut expr = self.logic_and()?;

        while self.r#match(TokenKind::Or) {
            let operator = self.previous().clone().lexeme;
            let right = self.logic_and()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            }
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> ExprRes<'a> {
        let mut expr = self.equality()?;

        while self.r#match(TokenKind::And) {
            let operator = self.previous().clone().lexeme;
            let right = self.equality()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ExprRes<'a> {
        let mut expr = self.range()?;

        while self.match_any(&[TokenKind::Ne, TokenKind::Eq]) {
            let operator = self.previous().lexeme;
            let right = self.equality()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            }
        }

        Ok(expr)
    }

    /// Parses a range expression.
    fn range(&mut self) -> ExprRes<'a> {
        if self.r#match(TokenKind::Range) {
            let operator = self.previous().lexeme;
            let mut step = 1 as Number;
            let mut start = 0 as Number;
            let mut end = self.as_number(
                self.consume(
                    TokenKind::Number(Number::default()),
                    "Expected a range stop value.",
                )?
                .kind,
            );
            if self.r#match(TokenKind::Number(Number::default())) {
                start = end;
                end = self.as_number(self.previous().kind);
            };
            if self.r#match(TokenKind::Number(Number::default())) {
                step = self.as_number(self.previous().kind);
            }
            Ok(Expr {
                kind: ExprKind::Range(Ptr::new(Range { start, end, step })),
                comment: None,
            })
        } else {
            self.comparison()
        }
    }

    fn comparison(&mut self) -> ExprRes<'a> {
        let mut expr = self.addition()?;

        while self.match_any(&[TokenKind::Lt, TokenKind::Le, TokenKind::Gt, TokenKind::Ge]) {
            let operator = self.previous().lexeme;
            let right = self.addition()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            };
        }

        Ok(expr)
    }

    fn addition(&mut self) -> ExprRes<'a> {
        let mut expr = self.multiplication()?;

        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous().lexeme;
            let right = self.multiplication()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            }
        }

        Ok(expr)
    }

    /// Parses a multiplication expression.
    fn multiplication(&mut self) -> ExprRes<'a> {
        let mut expr = self.exponentiation()?;

        while self.match_any(&[TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous().lexeme;
            let right = self.exponentiation()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            }
        }

        Ok(expr)
    }

    fn exponentiation(&mut self) -> ExprRes<'a> {
        let mut expr = self.unary()?;

        while self.r#match(TokenKind::Pow) {
            let operator = self.previous().lexeme;
            let right = self.exponentiation()?;
            expr = Expr {
                kind: ExprKind::Binary(Ptr::new(expr), operator, Ptr::new(right)),
                comment: None,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ExprRes<'a> {
        let expr = if self.match_any(&[TokenKind::Minus, TokenKind::Not, TokenKind::Ellipsis]) {
            let operator = self.previous().lexeme;
            let value = self.primary()?;
            Expr {
                kind: ExprKind::Unary(operator, Ptr::new(value)),
                comment: None,
            }
        } else {
            self.call()?
        };

        Ok(expr)
    }

    fn call(&mut self) -> ExprRes<'a> {
        let span = self.previous().span;
        let mut expr = self.primary()?;

        while self.match_any(&[TokenKind::LeftParen, TokenKind::Dot]) {
            expr = if self.previous().kind == TokenKind::LeftParen {
                self.finish_call(expr)
            } else {
                self.finish_attr(expr)
            }?;
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ExprRes<'a> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Nil
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Number(_)
            | TokenKind::StringLiteral => ExprKind::Literal(token.lexeme),
            TokenKind::LeftParen => ExprKind::Grouping(self.expression()?),
        }
    }

    fn finish_call(&mut self, callee: Expr<'a>) -> ExprRes<'a> {
        let args = if !self.check(&TokenKind::RightParen) {
            let args = self.arguments()?;
            let closer = self
                .consume(
                    TokenKind::RightParen,
                    "Expected a ')' after the arguments list.",
                )?
                .span;
            args
        } else {
            vec![]
        };
        Ok(Expr {
            kind: ExprKind::Call(Ptr::new(callee), args),
            comment: None,
        })
    }

    fn finish_attr(&mut self, obj: Expr<'a>) -> ExprRes<'a> {
        let attr = self.consume_identifier("Expected an attribute name after the dot.")?;
        Ok(Expr {
            kind: ExprKind::Get(Ptr::new(obj), attr.lexeme),
            comment: None,
        })
    }

    fn arguments(&mut self) -> Result<Vec<Expr<'a>>, ParseError> {
        let mut args = vec![];

        if !self.check(&TokenKind::RightParen) {
            let expr = self.expression()?;
            args.push(expr);
        }

        while self.r#match(TokenKind::Comma) {
            if self.check(&TokenKind::RightParen) {
                break;
            }
            args.push(self.expression()?);
        }

        Ok(args)
    }

    #[inline]
    fn consume_identifier<S: Into<String>>(&mut self, msg: S) -> Result<&Token<'a>, ParseError> {
        self.consume(TokenKind::Identifier, msg)
    }

    fn consume<S: Into<String>>(
        &mut self,
        kind: TokenKind,
        msg: S,
    ) -> Result<&Token<'a>, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(self.peek().span, msg))
        }
    }

    fn r#match(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(&kind) {
                self.advance();
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
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(kind) == std::mem::discriminant(&self.previous.kind)
    }

    fn advance(&mut self) -> &Token<'a> {
        self.previous = self
            .lexer
            .next()
            .map(|kind| Token::new(self.lexer.slice(), self.lexer.span(), kind))
            .unwrap_or(self.eof);
        self.current = self
            .lexer
            .next()
            .map(|kind| Token::new(self.lexer.slice(), self.lexer.span(), kind))
            .unwrap_or(self.eof);
        &self.previous
    }

    #[inline(always)]
    fn as_number(&self, value: TokenKind) -> Number {
        match value {
            TokenKind::Number(value) => value,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.current.kind == TokenKind::EOF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let source = r#"
        // We can disable the commands that require an API that is not available.
        /* 
        A multi-line comment
        */
/*
let a =  -1;
let b = 2 + 2;
let c = 3 * 3;
let d = 4 / 4;
let e = 4 // 4;
let f = 5 ** 5;
let g = 6 % 7;
// let h = (7 << 8) & ~(9 >> 10) | 3;
let i = true != false and 3 < 4 or 5 >= 6 or 3 <= 7 and 10 > 2;

let arr = [1, 2, 3];
let dict = {1: 2, 3: 4};

print(len(arr));
print(len(dict));"#;

        let lexer = Token::lexer(source);
        println!("{:#?}", parse(lexer));
    }
}
