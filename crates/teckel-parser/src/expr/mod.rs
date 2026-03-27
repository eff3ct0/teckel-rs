//! Expression language parser (Section 9).
//!
//! Implements a recursive descent parser using winnow, following the EBNF
//! grammar defined in the Teckel spec §9.1 and Appendix A.3.

mod parser;

pub use parser::parse_expression;
