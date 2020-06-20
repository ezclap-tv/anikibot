#[macro_export]
macro_rules! expr {
    ($self:ident, $kind:expr) => {{
        use $crate::dsl::ast::*;
        Expr {
            kind: $kind,
            comments: vec![],
        }
    }};
}

#[macro_export]
macro_rules! stmt {
    ($self:ident, $kind:expr) => {{
        use $crate::dsl::ast::*;
        Stmt {
            kind: $kind,
            comments: $self.comments.drain(..).collect(),
        }
    }};
}
