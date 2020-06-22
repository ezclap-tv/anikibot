//! This module implements the code generation stage of the `PPGA` to `Lua` transpiler.
//!
//! Usage example:
//!
//! ```
//! extern crate ppga;
//! use ppga::{frontend::{Parser, lexer}, codegen::emit_lua, config::PPGAConfig};
//!
//! let ast = Parser::with_config(
//!     PPGAConfig::default().disable_std(),
//!     lexer("fn f() { return 5; }")
//! ).parse().unwrap();
//! let output = emit_lua(&ast);
//! assert_eq!(output,
//! r#"local function f()
//!     return (5)
//! end"#);
//! ```
use crate::codegen::snippets::SNIPPETS;
use crate::{codegen::code_builder::*, config::PPGAConfig, frontend::ast::*};

/// Transpiles the given AST object into Lua.

/// ```
/// # extern crate ppga;
/// use ppga::{frontend::{Parser, lexer}, codegen::emit_lua, config::PPGAConfig};
///
/// let ast = Parser::with_config(
///     PPGAConfig::default().disable_std(),
///     lexer("fn f() { return 5; }")
/// ).parse().unwrap();
/// let output = emit_lua(&ast);
/// assert_eq!(output,
/// r#"local function f()
///     return (5)
/// end"#);
/// ```
pub fn emit_lua<'a>(ast: &AST<'a>) -> String {
    let mut code = CodeBuilder::new(ast.config.indent_size);
    if ast.config.include_ppga_std {
        code.push("-- PPGA STD SYMBOLS");
        for snippet in SNIPPETS.iter() {
            code.push(snippet.to_owned());
        }
        code.push("-- END PPGA STD SYMBOLS\n\n");
    }
    for stmt in &ast.stmts {
        code.push(stmt_to_lua(&stmt, &ast.config, 0).trim_end());
    }
    code.build()
}

/// Transpiles a single PPGA statement into a Lua statement.
///
/// ```
/// # extern crate ppga;
/// use ppga::{frontend::ast::*, codegen::stmt_to_lua, config::PPGAConfig};
///
/// let var = Expr::new(ExprKind::Variable("x"));
/// let expr = Ptr::new(Expr::new(ExprKind::Unary("-", Ptr::new(var.clone()))));
/// let stmt = Stmt::new(StmtKind::Assignment(vec![var], "=", expr));
/// let result = stmt_to_lua(&stmt, &PPGAConfig::default().disable_std(), 0);
/// assert_eq!(result, "x = -(x)");
/// ```
pub fn stmt_to_lua<'a>(stmt: &Stmt<'a>, config: &PPGAConfig, depth: usize) -> String {
    let mut code = CodeBuilder::new(config.indent_size);
    for comment in &stmt.comments {
        match comment {
            Comment::Regular(c) => {
                code.push_and_pad(format!("--{}", c), depth);
            }
            Comment::Multiline(c) => {
                code.append(format!("--[[{}--]]", c));
            }
        }
    }

    match &stmt.kind {
        StmtKind::If(cond, then, r#else) => {
            code.push_and_pad(
                format!("if {} then", expr_to_lua(&cond, config, depth)),
                depth,
            );
            code.push(stmt_to_lua(&then, config, depth));
            match r#else.as_ref().map(|s| &s.kind) {
                Some(StmtKind::If(_, _, _)) => {
                    // Pop the `end` of the previous block
                    let last = code.last_mut().unwrap();
                    for _ in 0..3 {
                        last.pop();
                    }
                    code.append(format!(
                        "else{}",
                        stmt_to_lua(r#else.as_ref().unwrap(), config, depth)
                    ));
                }
                Some(_) => {
                    // Pop the `end` of the previous block
                    let last = code.last_mut().unwrap();
                    for _ in 0..3 {
                        last.pop();
                    }
                    code.append("else")
                        .push(stmt_to_lua(r#else.as_ref().unwrap(), config, depth));
                }
                _ => (),
            }
        }
        StmtKind::For(r#for) => {
            code.push_and_pad("for ", depth)
                .append(match &r#for.condition {
                    ForCondition::Range(range) => {
                        format!("i = {}, {}, {}", range.start, range.end, range.step)
                    }
                    ForCondition::Exprs(exprs) => format!(
                        "{} in {}",
                        r#for
                            .vars
                            .iter()
                            // FIXME: this is gonna miss the comments attached to the param
                            .map(|v| match v.kind {
                                ExprKind::Param(v, _) => v,
                                _ => unreachable!(),
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                        {
                            let args = exprs
                                .iter()
                                .map(|e| expr_to_lua(e, config, depth))
                                .collect::<Vec<_>>()
                                .join(", ");
                            if exprs.len() == 1 && r#for.vars.len() == 2 {
                                format!(
                                    "{}({})",
                                    if r#for.is_fori { "ipairs" } else { "pairs" },
                                    args
                                )
                            } else {
                                args
                            }
                        }
                    ),
                })
                .append(" do")
                .push_and_pad(stmt_to_lua(&r#for.body, config, depth), depth);
        }
        StmtKind::While(cond, body) => {
            code.push_and_pad(
                format!("while {} do", expr_to_lua(&cond, config, depth)),
                depth,
            )
            .push_and_pad(stmt_to_lua(&body, config, depth), depth);
        }
        StmtKind::Block(body, is_standalone) => {
            if *is_standalone {
                code.push_and_pad("do", depth);
            }
            for stmt in body {
                code.push(stmt_to_lua(stmt, config, depth + 1));
            }
            code.push_and_pad("end", depth);
        }
        StmtKind::StmtSequence(stmts) => {
            for stmt in stmts {
                code.push(stmt_to_lua(stmt, config, depth));
            }
        }
        StmtKind::Return(values) => {
            code.push_and_pad(
                format!(
                    "return {}",
                    values
                        .iter()
                        .map(|v| match &v.kind {
                            ExprKind::Unary("...", _) => {
                                expr_to_lua(&v, config, depth)
                            }
                            _ => format!("({})", expr_to_lua(&v, config, depth)),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                depth,
            );
        }
        StmtKind::ExprStmt(expr) => {
            code.push_and_pad(expr_to_lua(&expr, config, depth), depth);
        }
        StmtKind::Assignment(vars, op, value) => {
            code.push_and_pad(
                format!(
                    "{} {} {}",
                    vars.iter()
                        .map(|v| expr_to_lua(&v, config, depth))
                        .collect::<Vec<_>>()
                        .join(", "),
                    match *op {
                        "**=" => format!("= {} {}", expr_to_lua(&value, config, depth), "^"),
                        rest => rest.to_string(),
                    },
                    expr_to_lua(&value, config, depth)
                ),
                depth,
            );
        }
        StmtKind::FuncDecl(kind, r#fn) => {
            code.push_and_pad(fn_to_lua(r#fn, *kind, config, depth), depth);
        }
        StmtKind::VarDecl(kind, names, value) => {
            code.push_and_pad(
                format!(
                    "{}{}{}",
                    match kind {
                        VarKind::Local => "local ",
                        VarKind::Global => "",
                    },
                    names
                        .iter()
                        .map(|v| match v {
                            VarName::Borrowed(b) => b.to_string(),
                            VarName::Owned(o) => o.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                    match value {
                        Some(ref expr) => format!(" = {}", expr_to_lua(expr, config, depth)),
                        None => String::new(),
                    }
                ),
                depth,
            );
        }
        StmtKind::Break => {
            code.push_and_pad("break", depth);
        }
        StmtKind::NewLine(n) => {
            code.append("\n".repeat(*n));
        }
    }

    code.build()
}

/// Transpiles a single PPGA expression into a Lua expression.
///
/// ```
/// # extern crate ppga;
/// use ppga::{frontend::ast::*, codegen::expr_to_lua, config::PPGAConfig};
///
/// let expr = Expr::new(ExprKind::Unary("-", Ptr::new(Expr::new(ExprKind::Variable("x")))));
/// let result = expr_to_lua(&expr, &PPGAConfig::default().disable_std(), 0);
/// assert_eq!(result, "-(x)");
/// ```
pub fn expr_to_lua<'a>(expr: &Expr<'a>, config: &PPGAConfig, depth: usize) -> String {
    let mut code = CodeBuilder::new(config.indent_size);

    for comment in &expr.comments {
        match comment {
            Comment::Regular(c) => {
                code.push_and_pad(format!("--{}", c), depth);
            }
            Comment::Multiline(c) => {
                code.append(format!("--[[{}--]]", c));
            }
        }
    }

    match &expr.kind {
        ExprKind::LuaBlock(lua) => {
            let indent = " ".repeat(DEFAULT_INDENT_SIZE);
            let lines = lua
                .split("\n")
                .map(|line| line.strip_prefix(&indent).unwrap_or(line))
                .collect::<Vec<_>>();
            code.append(lines.join("\n"));
        }
        ExprKind::Literal(lit, is_str) => {
            code.append(if *is_str {
                format!("\"{}\"", lit)
            } else {
                format!("{}", lit)
            });
        }
        ExprKind::Variable(v) | ExprKind::Param(v, _) => {
            code.append(v.to_string());
        }
        ExprKind::GeneratedVariable(v) => {
            code.append(v.to_string());
        }
        ExprKind::FString(frags) => {
            code.append(
                frags
                    .iter()
                    .map(|frag| match frag.kind {
                        ExprKind::Literal(_, true) => expr_to_lua(frag, config, depth),
                        _ => format!("tostring({})", expr_to_lua(frag, config, depth)),
                    })
                    .collect::<Vec<_>>()
                    .join(" .. "),
            );
        }
        ExprKind::Get(obj, attr, is_static) => {
            code.append(format!(
                "{}{}{}",
                expr_to_lua(&obj, config, depth),
                if *is_static { ":" } else { "." },
                attr,
            ));
        }
        ExprKind::GetItem(obj, key) => {
            code.append(format!(
                "{}[{}]",
                expr_to_lua(&obj, config, depth),
                expr_to_lua(&key, config, depth)
            ));
        }
        ExprKind::Call(callee, args) => {
            code.append(format!(
                "{}({})",
                expr_to_lua(&callee, config, depth),
                args.iter()
                    .map(|a| expr_to_lua(&a, config, depth))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        ExprKind::Len(obj) => {
            code.append(format!("#({})", expr_to_lua(&obj, config, depth)));
        }
        ExprKind::Unary(op, obj) => {
            code.append(match *op {
                "..." => expr_to_lua(&obj, config, depth),
                _ => format!("{}({})", op, expr_to_lua(&obj, config, depth)),
            });
        }
        ExprKind::Grouping(obj) => {
            code.append(format!("({})", expr_to_lua(&obj, config, depth)));
        }
        ExprKind::Binary(l, op, r) => {
            code.append(format!(
                "{} {} {}",
                expr_to_lua(&l, config, depth),
                match *op {
                    "\\" => "//",
                    "**" => "^",
                    "!=" => "~=",
                    rest => rest,
                },
                expr_to_lua(&r, config, depth)
            ));
        }
        ExprKind::ArrayLiteral(args) => {
            code.append(format!(
                "{{{}}}",
                args.iter()
                    .enumerate()
                    .map(|(i, a)| format!("[{}] = {}", i, expr_to_lua(a, config, depth)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        ExprKind::DictLiteral(pairs) => {
            code.append(format!(
                "{{{}{}{}",
                "\n".repeat(pairs.len().min(1).max(0)),
                pairs
                    .iter()
                    .map(|(k, v)| pad(
                        format!(
                            "[{}] = {}",
                            expr_to_lua(&k, config, depth),
                            expr_to_lua(&v, config, depth)
                        ),
                        depth + 1,
                        DEFAULT_INDENT_SIZE,
                    ))
                    .collect::<Vec<_>>()
                    .join(",\n"),
                if pairs.is_empty() {
                    String::from("}")
                } else {
                    format!("\n{}", pad("}", depth, DEFAULT_INDENT_SIZE))
                }
            ));
        }
        ExprKind::Lambda(r#fn) => {
            code.append(fn_to_lua(&r#fn, VarKind::Local, config, depth + 1));
        }
        ExprKind::NewLine => {
            code.append("\n");
        }
    }

    code.build()
}

/// Transpiles a single PPGA function into a Lua function.
pub fn fn_to_lua<'a>(
    r#fn: &Function<'a>,
    kind: VarKind,
    config: &PPGAConfig,
    depth: usize,
) -> String {
    CodeBuilder::new(config.indent_size)
        .push(format!(
            "{}function {}({})",
            if r#fn.name.is_some() && kind == VarKind::Local {
                "local "
            } else {
                ""
            },
            r#fn.name.unwrap_or(""),
            r#fn.params
                .iter()
                .map(|p| match &expr_to_lua(&p, config, depth)[..] {
                    "@" => "...".to_string(),
                    rest => rest.to_owned(),
                })
                .collect::<Vec<_>>()
                .join(", "),
        ))
        .push(stmt_to_lua(&r#fn.body, config, depth))
        .build()
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::{lexer, parser::*};
    use logos::Logos;

    #[test]
    fn test_codegen() {
        let source = r#"fn some_api_request() {
    return "ok", nil;
}

let ok = some_api_request()?;
print(f"ok: {ok}");

let ok2 = some_api_request() err { return "handled", nil; }?;
print(f"ok2: {ok2}");

let val = nil;
let res = val ?? 42;
print(f"res: {res}");"#;
        let expected = r#"-- PPGA STD SYMBOLS
local function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end
local function __PPGA_INTERNAL_HANDLE_ERR(cb, ...)
    local ok, err = ...
    if err ~= nil then
        ok, err = cb(err)
    end
    return (ok), (err)
end
-- END PPGA STD SYMBOLS


local function some_api_request()
    return ("ok"), (nil)
end

local ok = nil
do
    local _ok_L4S76, _err_L4S76 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, some_api_request())
    if _err_L4S76 ~= nil then
        return (_err_L4S76)
    end
    ok = _ok_L4S76
end
print("ok: " .. tostring(ok))

local ok2 = nil
do
    local _ok_L7S159, _err_L7S159 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            return ("handled"), (nil)
        end, some_api_request())
    if _err_L7S159 ~= nil then
        return (_err_L7S159)
    end
    ok2 = _ok_L7S159
end
print("ok2: " .. tostring(ok2))

local val = nil
local res = __PPGA_INTERNAL_DEFAULT(val, 42)
print("res: " .. tostring(res))"#;
        let parser = Parser::new(lexer(source));
        let ast = parser.parse().map_err(|e| e.report_all()).unwrap();
        let lua = emit_lua(&ast);
        assert_eq!(expected, lua);
    }
}
