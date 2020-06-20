use super::{
    ast::*,
    code_builder::{pad, CodeBuilder, DEFAULT_INDENT_SIZE},
};

pub fn emit_lua<'a>(ast: &AST<'a>) -> String {
    let mut code = CodeBuilder::default();
    for stmt in &ast.stmts {
        code.push(stmt_to_lua(&stmt, 0).trim_end());
    }
    code.build()
}

pub fn stmt_to_lua<'a>(stmt: &Stmt<'a>, depth: usize) -> String {
    let mut code = CodeBuilder::default();
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
            code.push_and_pad(format!("if {} then", expr_to_lua(&cond, depth)), depth);
            code.push(stmt_to_lua(&then, depth));
            match r#else.as_ref().map(|s| &s.kind) {
                Some(StmtKind::If(_, _, _)) => {
                    // Pop the `end` of the previous block
                    let last = code.last_mut().unwrap();
                    for _ in 0..3 {
                        last.pop();
                    }
                    code.append(format!(
                        "else{}",
                        stmt_to_lua(r#else.as_ref().unwrap(), depth)
                    ));
                }
                Some(_) => {
                    // Pop the `end` of the previous block
                    let last = code.last_mut().unwrap();
                    for _ in 0..3 {
                        last.pop();
                    }
                    code.append("else")
                        .push(stmt_to_lua(r#else.as_ref().unwrap(), depth));
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
                                .map(|e| expr_to_lua(e, depth))
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
                .push_and_pad(stmt_to_lua(&r#for.body, depth), depth);
        }
        StmtKind::While(cond, body) => {
            code.push_and_pad(format!("while {} do", expr_to_lua(&cond, depth)), depth)
                .push_and_pad(stmt_to_lua(&body, depth), depth);
        }
        StmtKind::Block(body, is_standalone) => {
            if *is_standalone {
                code.push_and_pad("do", depth);
            }
            for stmt in body {
                code.push(stmt_to_lua(stmt, depth + 1));
            }
            code.push_and_pad("end", depth);
        }
        StmtKind::Return(values) => {
            code.push_and_pad(
                format!(
                    "return {}",
                    values
                        .iter()
                        .map(|v| match &v.kind {
                            ExprKind::Unary("...", _) => {
                                expr_to_lua(&v, depth)
                            }
                            _ => format!("({})", expr_to_lua(&v, depth)),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                depth,
            );
        }
        StmtKind::ExprStmt(expr) => {
            code.push_and_pad(expr_to_lua(&expr, depth), depth);
        }
        StmtKind::Assignment(obj, op, value) => {
            code.push_and_pad(
                format!(
                    "{} {} {}",
                    expr_to_lua(&obj, depth),
                    match *op {
                        "**=" => format!("= {} {}", expr_to_lua(&value, depth), "^"),
                        rest => rest.to_string(),
                    },
                    expr_to_lua(&value, depth)
                ),
                depth,
            );
        }
        StmtKind::FuncDecl(kind, r#fn) => {
            code.push_and_pad(fn_to_lua(r#fn, *kind, depth), depth);
        }
        StmtKind::VarDecl(kind, name, value) => {
            code.push_and_pad(
                format!(
                    "{}{}{}",
                    match kind {
                        VarKind::Local => "local ",
                        VarKind::Global => "",
                    },
                    name,
                    match value {
                        Some(ref expr) => format!(" = {}", expr_to_lua(expr, depth)),
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

pub fn expr_to_lua<'a>(expr: &Expr<'a>, depth: usize) -> String {
    let mut code = CodeBuilder::default();

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
        ExprKind::FString(frags) => {
            code.append(
                frags
                    .iter()
                    .map(|frag| match frag.kind {
                        ExprKind::Literal(_, true) => expr_to_lua(frag, depth),
                        _ => format!("tostring({})", expr_to_lua(frag, depth)),
                    })
                    .collect::<Vec<_>>()
                    .join(" .. "),
            );
        }
        ExprKind::Get(obj, attr, is_static) => {
            code.append(format!(
                "{}{}{}",
                expr_to_lua(&obj, depth),
                if *is_static { ":" } else { "." },
                attr,
            ));
        }
        ExprKind::GetItem(obj, key) => {
            code.append(format!(
                "{}[{}]",
                expr_to_lua(&obj, depth),
                expr_to_lua(&key, depth)
            ));
        }
        ExprKind::Call(callee, args) => {
            code.append(format!(
                "{}({})",
                expr_to_lua(&callee, depth),
                args.iter()
                    .map(|a| expr_to_lua(&a, depth))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        ExprKind::Len(obj) => {
            code.append(format!("#({})", expr_to_lua(&obj, depth)));
        }
        ExprKind::Unary(op, obj) => {
            code.append(match *op {
                "..." => expr_to_lua(&obj, depth),
                _ => format!("{}({})", op, expr_to_lua(&obj, depth)),
            });
        }
        ExprKind::Grouping(obj) => {
            code.append(format!("({})", expr_to_lua(&obj, depth)));
        }
        ExprKind::Binary(l, op, r) => {
            code.append(format!(
                "{} {} {}",
                expr_to_lua(&l, depth),
                match *op {
                    "\\" => "//",
                    "**" => "^",
                    "!=" => "~=",
                    rest => rest,
                },
                expr_to_lua(&r, depth)
            ));
        }
        ExprKind::ArrayLiteral(args) => {
            code.append(format!(
                "{{{}}}",
                args.iter()
                    .enumerate()
                    .map(|(i, a)| format!("[{}] = {}", i, expr_to_lua(a, depth)))
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
                        format!("[{}] = {}", expr_to_lua(&k, depth), expr_to_lua(&v, depth)),
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
            code.append(fn_to_lua(&r#fn, VarKind::Local, depth));
        }
        ExprKind::NewLine => {
            code.append("\n");
        }
    }

    code.build()
}

fn fn_to_lua<'a>(r#fn: &Function<'a>, kind: VarKind, depth: usize) -> String {
    CodeBuilder::default()
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
                .map(|p| expr_to_lua(&p, depth))
                .collect::<Vec<_>>()
                .join(", "),
        ))
        .push(stmt_to_lua(&r#fn.body, depth))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{lexer::*, parser::*};
    use logos::Logos;

    #[test]
    fn test_codegen() {
        let source = r#"// lel
a;
let a =  -1;
let b = 2 + 2;
let c = 3 * 3;
let d = 4 / 4;
let e = 4 \ 4;
let f = 5 ** 5;
let g = 6 % 7;
let i = true != false and 3 < 4 or 5 >= 6 or 3 <= 7 and 10 > 2;

let arr = [1, 2, 3];
let dict = {1 = 2, 3 = 4};

print(len(arr));
print(len(dict));
        "#;
        let parser = Parser::new(TokenKind::lexer(source));
        let ast = parser.parse().map_err(|e| e.report_all()).unwrap();
        let lua = emit_lua(&ast);
        assert!(false, "{}", lua);
    }
}
