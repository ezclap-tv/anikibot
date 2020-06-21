#[macro_export]
macro_rules! expr {
    ($self:ident, $kind:expr) => {{
        use $crate::frontend::ast::*;
        Expr {
            kind: $kind,
            comments: vec![],
        }
    }};
}

#[macro_export]
macro_rules! stmt {
    ($self:ident, $kind:expr) => {{
        use $crate::frontend::ast::*;
        Stmt {
            kind: $kind,
            comments: $self.comments.drain(..).collect(),
        }
    }};
}

#[macro_export]
macro_rules! make_return {
    ($value:expr) => {{
        use $crate::frontend::ast::*;
        Stmt::new(StmtKind::Return(vec![$value]))
    }};
}

#[macro_export]
macro_rules! make_literal {
    ($lit:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Literal($lit, false))
    }};

    (str => $lit:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Literal($lit, true))
    }};
}

#[macro_export]
macro_rules! make_binary {
    ($left:expr, $op:expr, $right:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Binary($left, $op, $right))
    }};

    (alloc => $left:expr, $op:expr, $right:expr) => {{
        use $crate::frontend::ast::*;
        Ptr::new(Expr::new(ExprKind::Binary(
            Ptr::new($left),
            $op,
            Ptr::new($right),
        )))
    }};
}

#[macro_export]
macro_rules! make_block {
    ($standalone: tt, $( $v:expr ),*) => {{
        use $crate::frontend::ast::*;
        Stmt::new(StmtKind::Block(vec![$( $v ),*], $standalone))
    }};
}

#[macro_export]
macro_rules! make_var {
    ($name:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Variable($name))
    }};
}

#[macro_export]
macro_rules! owned_var {
    ($name:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::GeneratedVariable($name.into()))
    }};
}

#[macro_export]
macro_rules! make_expr_stmt {
    ($value:expr) => {{
        use $crate::frontend::ast::*;
        Stmt::new(StmtKind::ExprStmt($value))
    }};
    (alloc => $value:expr) => {{
        use $crate::frontend::ast::*;
        Stmt::new(StmtKind::ExprStmt(Ptr::new($value)))
    }};
}

#[macro_export]
macro_rules! make_get {
    ($obj:expr, $attr:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Get($obj, $attr, false))
    }};
    (alloc => $obj:expr, $attr:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Get(Ptr::new($obj), $attr, false))
    }};
    ($obj:expr, colon => $attr:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Get($obj, $attr, true))
    }};
    (alloc => $obj:expr, colon => $attr:expr) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Get(Ptr::new($obj), $attr, true))
    }};
}

#[macro_export]
macro_rules! make_call {
    ($callee: expr, $( $v:expr ),*) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Call($callee, vec![$( $v ),*]))
    }};

    (alloc => $callee: expr, $( $v:expr ),*) => {{
        use $crate::frontend::ast::*;
        Expr::new(ExprKind::Call(Ptr::new($callee), vec![$( $v ),*]))
    }};
}
