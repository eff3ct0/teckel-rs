//! Expression AST for the Teckel expression language (Section 9).

use crate::types::{FrameType, NullOrdering, SortDirection};

/// Window frame specification for inline window expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrame {
    pub frame_type: FrameType,
    pub start: FrameBound,
    pub end: FrameBound,
}

/// Window frame boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBound {
    UnboundedPreceding,
    UnboundedFollowing,
    CurrentRow,
    Preceding(Box<Expr>),
    Following(Box<Expr>),
}

/// A parsed Teckel expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value.
    Literal(Literal),
    /// A column reference, optionally qualified: `table.column`.
    ColumnRef {
        table: Option<String>,
        column: String,
    },
    /// A binary operation: `left op right`.
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    /// A unary operation: `op expr`.
    UnaryOp { op: UnaryOp, expr: Box<Expr> },
    /// A function call: `name([DISTINCT] args...)`.
    FunctionCall {
        name: String,
        args: Vec<Expr>,
        distinct: bool,
    },
    /// A CASE expression.
    Case {
        when_clauses: Vec<(Expr, Expr)>,
        else_clause: Option<Box<Expr>>,
    },
    /// A CAST expression: `CAST(expr AS type)`.
    Cast {
        expr: Box<Expr>,
        data_type: String,
    },
    /// `expr IS [NOT] NULL`.
    IsNull { expr: Box<Expr>, negated: bool },
    /// `expr [NOT] IN (list...)`.
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    /// `expr [NOT] BETWEEN low AND high`.
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
    /// `expr [NOT] LIKE pattern`.
    Like {
        expr: Box<Expr>,
        pattern: Box<Expr>,
        negated: bool,
    },
    /// `expr as alias` — aliased expression.
    Alias { expr: Box<Expr>, alias: String },
    /// Wildcard `*` — used in `count(*)`.
    Wildcard,
    /// `expr RLIKE pattern` — regex match.
    RLike {
        expr: Box<Expr>,
        pattern: Box<Expr>,
        negated: bool,
    },
    /// `left <=> right` — null-safe equality.
    NullSafeEq {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `left || right` — string concatenation.
    Concat {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `TRY_CAST(expr AS type)`.
    TryCast {
        expr: Box<Expr>,
        data_type: String,
    },
    /// `func() OVER (PARTITION BY ... ORDER BY ... frame)`.
    WindowExpr {
        function: Box<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<(Expr, SortDirection, NullOrdering)>,
        frame: Option<WindowFrame>,
    },
    /// Lambda expression: `x -> expr` or `(x, y) -> expr`.
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    /// Named argument in function call: `key => value`.
    NamedArg {
        name: String,
        value: Box<Expr>,
    },
    /// Nested field access: `expr.field`.
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    /// Subscript access: `expr[key]`.
    Subscript {
        expr: Box<Expr>,
        key: Box<Expr>,
    },
    /// Qualified star: `table.*`.
    QualifiedWildcard {
        qualifier: String,
    },
    /// Typed literal: `DATE '2025-01-01'`, `TIMESTAMP '...'`, etc.
    TypedLiteral {
        type_name: String,
        value: String,
    },
    /// Complex literal: `ARRAY(...)`, `MAP(...)`, `STRUCT(...)`, `NAMED_STRUCT(...)`.
    ComplexLiteral {
        constructor: String,
        args: Vec<Expr>,
    },
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Neq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    // Logical
    And,
    Or,
    /// String concatenation `||`
    StringConcat,
    /// Null-safe equality `<=>`
    NullSafeEq,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::LtEq => "<=",
            Self::GtEq => ">=",
            Self::And => "AND",
            Self::Or => "OR",
            Self::StringConcat => "||",
            Self::NullSafeEq => "<=>",
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Neg => "-",
            Self::Not => "NOT",
        };
        write!(f, "{s}")
    }
}
