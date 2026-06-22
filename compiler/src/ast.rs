use std::fmt;
use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

// === DECLARATIONS ===

#[derive(Debug, Clone)]
pub enum Decl {
    Var(VarDecl),
    Const(ConstDecl),
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Union(UnionDecl),
    Import(ImportDecl),
    Foreign(ForeignDecl),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub mutable: bool,
    pub type_: Option<Type>,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub type_: Option<Type>,
    pub init: Expr,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_extern: bool,
    pub foreign_lib: Option<String>,
    pub pub_: bool,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub backing_type: Option<Type>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct UnionDecl {
    pub name: String,
    pub tagged: bool,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct ForeignDecl {
    pub lib_name: String,
    pub lib_path: Option<String>,
    pub declarations: Vec<Decl>,
}

// === TYPES ===

#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
    Ptr(Box<Type>),
    ConstPtr(Box<Type>),
    NullablePtr(Box<Type>),
    ManyPtr(Box<Type>),
    Slice(Box<Type>),
    Array(u64, Box<Type>),
    Optional(Box<Type>),
    ErrorUnion(Box<Type>),
    Fn(Vec<Type>, Option<Box<Type>>),
    Inferred,
}

// === EXPRESSIONS ===

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    CharLit(char),
    BoolLit(bool),
    NullLit,
    Ident(String),
    Undefined,

    Block(Block),
    If(Box<Expr>, Block, Option<Box<Expr>>),
    For(Box<Expr>, Option<String>, Option<String>, Block),
    While(Box<Expr>, Block),
    Switch(Box<Expr>, Vec<SwitchArm>, Option<Block>),

    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),

    Call(Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>),

    StructInit(String, Vec<(String, Expr)>),
    ArrayLit(Vec<Expr>),

    Try(Box<Expr>),
    Catch(Box<Expr>, Option<String>, Block),

    Comptime(Block),
    When(Box<Expr>, Block, Option<Block>),
}

#[derive(Debug, Clone)]
pub struct SwitchArm {
    pub patterns: Vec<SwitchPattern>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum SwitchPattern {
    Expr(Expr),
    Range(Expr, Expr),
    Else,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Block {
    pub fn empty() -> Self {
        Block { stmts: vec![] }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Defer(Expr),
    Errdefer(Expr),
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    Not,
    Addr,
    Deref,
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
        }
    }
}
