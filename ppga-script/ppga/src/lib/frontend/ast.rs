use super::lexer::Number;
use crate::PPGAConfig;

/// The pointer type used to wrap Expr/Stmt nodes.
pub type Ptr<T> = Box<T>;
/// An Expr wrapped in the pointer type.
pub type ExprPtr<'a> = Ptr<Expr<'a>>;
/// An Stmt wrapped in the pointer type.
pub type StmtPtr<'a> = Ptr<Stmt<'a>>;

/// An abstract syntax tree that holds a list of statements and a [`PPGAConfig`].
///
/// [`PPGAConfig`]: crate::config::PPGAConfig
#[derive(Debug, Clone)]
pub struct AST<'a> {
    /// The list of top-level statements in the PPGA program.
    pub stmts: Vec<Stmt<'a>>,
    /// The config used used in some of the transpilation stages.
    pub config: PPGAConfig,
    #[doc(hidden)]
    pub comments: Vec<Comment<'a>>,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub start: Number,
    pub end: Number,
    pub step: Number,
}

#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: Option<&'a str>,
    pub params: Vec<Expr<'a>>,
    pub body: Stmt<'a>,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'a> {
    Len(ExprPtr<'a>),
    LuaBlock(&'a str),
    Literal(&'a str, bool),
    Variable(&'a str),
    GeneratedVariable(String),
    Param(&'a str, bool),
    FString(Vec<Expr<'a>>),
    Get(ExprPtr<'a>, &'a str, bool),
    GetItem(ExprPtr<'a>, ExprPtr<'a>),
    Call(ExprPtr<'a>, Vec<Expr<'a>>),
    Unary(&'a str, ExprPtr<'a>),
    Grouping(ExprPtr<'a>),
    Binary(ExprPtr<'a>, &'a str, ExprPtr<'a>),
    ArrayLiteral(Vec<Expr<'a>>),
    DictLiteral(Vec<(Expr<'a>, Expr<'a>)>),
    Lambda(Ptr<Function<'a>>),
    NewLine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarKind {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub enum ForCondition<'a> {
    Range(Ptr<Range>),
    Exprs(Vec<Expr<'a>>),
}
#[derive(Debug, Clone)]
pub struct ForLoop<'a> {
    pub is_fori: bool,
    pub vars: Vec<Expr<'a>>,
    pub condition: ForCondition<'a>,
    pub body: StmtPtr<'a>,
}

#[derive(Debug, Clone)]
pub enum VarName<'a> {
    Borrowed(&'a str),
    Owned(String),
}

#[derive(Debug, Clone)]
pub enum StmtKind<'a> {
    If(ExprPtr<'a>, StmtPtr<'a>, Option<StmtPtr<'a>>),
    For(ForLoop<'a>),
    While(ExprPtr<'a>, StmtPtr<'a>),
    Block(Vec<Stmt<'a>>, bool),
    /// Used for inserting multiple statements without generating a block.
    StmtSequence(Vec<Stmt<'a>>),
    Return(Vec<Expr<'a>>),
    ExprStmt(ExprPtr<'a>),
    Assignment(Vec<Expr<'a>>, &'a str, ExprPtr<'a>),
    FuncDecl(VarKind, Ptr<Function<'a>>),
    VarDecl(VarKind, Vec<VarName<'a>>, Option<ExprPtr<'a>>),
    Break,
    NewLine(usize),
}

#[derive(Debug, Clone)]
pub struct Stmt<'a> {
    pub kind: StmtKind<'a>,
    pub comments: Vec<Comment<'a>>,
}

impl<'a> Stmt<'a> {
    pub fn new(kind: StmtKind<'a>) -> Self {
        Self {
            kind,
            comments: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub comments: Vec<Comment<'a>>,
}

impl<'a> Expr<'a> {
    pub fn new(kind: ExprKind<'a>) -> Self {
        Self {
            kind,
            comments: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum Comment<'a> {
    Regular(&'a str),
    Multiline(&'a str),
}
