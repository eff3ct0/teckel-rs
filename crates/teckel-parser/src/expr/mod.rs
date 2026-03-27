// Expression language parser (Section 9).
//
// Phase 1: Expressions are treated as raw strings and passed to DataFusion's
// SQL parser at execution time. A full AST parser will be implemented in Phase 2
// using winnow for pre-validation and cross-backend portability.
//
// The planned AST types are:
//
// pub enum Expr {
//     Literal(Literal),
//     ColumnRef { table: Option<String>, column: String },
//     BinaryOp { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
//     UnaryOp { op: UnaryOp, expr: Box<Expr> },
//     FunctionCall { name: String, args: Vec<Expr>, distinct: bool },
//     Case { when_clauses: Vec<(Expr, Expr)>, else_clause: Option<Box<Expr>> },
//     Cast { expr: Box<Expr>, data_type: TeckelDataType },
//     IsNull { expr: Box<Expr>, negated: bool },
//     InList { expr: Box<Expr>, list: Vec<Expr>, negated: bool },
//     Between { expr: Box<Expr>, low: Box<Expr>, high: Box<Expr>, negated: bool },
//     Like { expr: Box<Expr>, pattern: Box<Expr>, negated: bool },
//     Alias { expr: Box<Expr>, alias: String },
// }
