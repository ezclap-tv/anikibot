use super::lexer::Number;
use logos::Span;

pub type Ptr<T> = Box<T>;
pub type ExprPtr<'a> = Ptr<Expr<'a>>;
pub type StmtPtr<'a> = Ptr<Stmt<'a>>;

pub struct AST<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub struct Range {
    start: Number,
    end: Number,
    step: Number,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'a> {
    Literal(&'a str),
    Get(ExprPtr<'a>, &'a str),
    Call(ExprPtr<'a>, Vec<Expr<'a>>),
    Range(Ptr<Range>),
    Binary(ExprPtr<'a>, &'a str, ExprPtr<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarKind {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub enum StmtKind<'a> {
    VarDecl(VarKind, &'a str, Option<ExprPtr<'a>>),
}

#[derive(Debug, Clone)]
pub struct Stmt<'a> {
    pub kind: StmtKind<'a>,
}

#[derive(Debug, Clone)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub comment: Option<Comment<'a>>,
}

#[derive(Debug, Clone)]
pub enum Comment<'a> {
    Regular(&'a str),
    Multiline(&'a str),
}
