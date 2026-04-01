//! Hand-rolled recursive descent parser for the Teckel expression language.
//!
//! Operator precedence (highest to lowest, per spec §9.2):
//! 1. (), function calls
//! 2. unary -
//! 3. *, /, %
//! 4. +, -
//! 5. =, !=, <>, <, >, <=, >=, IS, IN, BETWEEN, LIKE
//! 6. NOT
//! 7. AND
//! 8. OR
//! 9. AS (aliasing)

use teckel_model::expr::{BinaryOp, Expr, Literal, UnaryOp};

/// Parse a Teckel expression string into an AST.
pub fn parse_expression(input: &str) -> Result<Expr, String> {
    let mut p = Parser::new(input);
    let expr = p.expression()?;
    p.skip_ws();
    if p.pos < p.input.len() {
        return Err(format!(
            "unexpected trailing input at position {}: '{}'",
            p.pos,
            &p.input[p.pos..]
        ));
    }
    Ok(expr)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn expect_char(&mut self, c: char) -> Result<(), String> {
        self.skip_ws();
        match self.peek_char() {
            Some(ch) if ch == c => {
                self.advance(c.len_utf8());
                Ok(())
            }
            Some(ch) => Err(format!(
                "expected '{c}', found '{ch}' at position {}",
                self.pos
            )),
            None => Err(format!("expected '{c}', found end of input")),
        }
    }

    /// Try to consume a keyword (case-insensitive) followed by a word boundary.
    fn try_keyword(&mut self, kw: &str) -> bool {
        let rem = self.remaining();
        if rem.len() < kw.len() {
            return false;
        }
        if !rem[..kw.len()].eq_ignore_ascii_case(kw) {
            return false;
        }
        // Word boundary: next char must not be alphanumeric or underscore
        if rem.len() > kw.len() {
            let next = rem.as_bytes()[kw.len()];
            if next.is_ascii_alphanumeric() || next == b'_' {
                return false;
            }
        }
        self.advance(kw.len());
        true
    }

    fn peek_keyword(&self, kw: &str) -> bool {
        let rem = self.remaining();
        if rem.len() < kw.len() {
            return false;
        }
        if !rem[..kw.len()].eq_ignore_ascii_case(kw) {
            return false;
        }
        if rem.len() > kw.len() {
            let next = rem.as_bytes()[kw.len()];
            if next.is_ascii_alphanumeric() || next == b'_' {
                return false;
            }
        }
        true
    }

    // ── Top-level ────────────────────────────────────────────

    fn expression(&mut self) -> Result<Expr, String> {
        let expr = self.or_expr()?;
        self.skip_ws();
        if self.try_keyword("AS") {
            self.skip_ws();
            let alias = self.identifier()?;
            Ok(Expr::Alias {
                expr: Box::new(expr),
                alias,
            })
        } else {
            Ok(expr)
        }
    }

    // ── OR ───────────────────────────────────────────────────

    fn or_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.and_expr()?;
        loop {
            self.skip_ws();
            if self.try_keyword("OR") {
                self.skip_ws();
                let right = self.and_expr()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::Or,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    // ── AND ──────────────────────────────────────────────────

    fn and_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.not_expr()?;
        loop {
            self.skip_ws();
            if self.try_keyword("AND") {
                self.skip_ws();
                let right = self.not_expr()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::And,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    // ── NOT ──────────────────────────────────────────────────

    fn not_expr(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.try_keyword("NOT") {
            self.skip_ws();
            let expr = self.comparison()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            })
        } else {
            self.comparison()
        }
    }

    // ── Comparison ───────────────────────────────────────────

    fn comparison(&mut self) -> Result<Expr, String> {
        let left = self.addition()?;
        self.skip_ws();

        // IS [NOT] NULL
        if self.try_keyword("IS") {
            self.skip_ws();
            let negated = self.try_keyword("NOT");
            if negated {
                self.skip_ws();
            }
            if !self.try_keyword("NULL") {
                return Err(format!("expected NULL after IS at position {}", self.pos));
            }
            return Ok(Expr::IsNull {
                expr: Box::new(left),
                negated,
            });
        }

        // [NOT] IN / BETWEEN / LIKE
        let saved = self.pos;
        let negated = self.try_keyword("NOT");
        if negated {
            self.skip_ws();
        }

        if self.try_keyword("IN") {
            self.skip_ws();
            self.expect_char('(')?;
            let list = self.expression_list()?;
            self.skip_ws();
            self.expect_char(')')?;
            return Ok(Expr::InList {
                expr: Box::new(left),
                list,
                negated,
            });
        }

        if self.try_keyword("BETWEEN") {
            self.skip_ws();
            let low = self.addition()?;
            self.skip_ws();
            if !self.try_keyword("AND") {
                return Err(format!("expected AND in BETWEEN at position {}", self.pos));
            }
            self.skip_ws();
            let high = self.addition()?;
            return Ok(Expr::Between {
                expr: Box::new(left),
                low: Box::new(low),
                high: Box::new(high),
                negated,
            });
        }

        if self.try_keyword("LIKE") {
            self.skip_ws();
            let pattern = self.primary()?;
            return Ok(Expr::Like {
                expr: Box::new(left),
                pattern: Box::new(pattern),
                negated,
            });
        }

        // Undo NOT consumption if no IN/BETWEEN/LIKE followed
        if negated {
            self.pos = saved;
        }

        // Comparison operators
        if let Some(op) = self.try_comp_op() {
            self.skip_ws();
            let right = self.addition()?;
            return Ok(Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    #[allow(clippy::if_same_then_else)]
    fn try_comp_op(&mut self) -> Option<BinaryOp> {
        let rem = self.remaining();
        let (op, len) = if rem.starts_with("<=") {
            (BinaryOp::LtEq, 2)
        } else if rem.starts_with(">=") {
            (BinaryOp::GtEq, 2)
        } else if rem.starts_with("<>") {
            (BinaryOp::Neq, 2)
        } else if rem.starts_with("!=") {
            (BinaryOp::Neq, 2)
        } else if rem.starts_with("==") {
            (BinaryOp::Eq, 2)
        } else if rem.starts_with('=') {
            (BinaryOp::Eq, 1)
        } else if rem.starts_with('<') {
            (BinaryOp::Lt, 1)
        } else if rem.starts_with('>') {
            (BinaryOp::Gt, 1)
        } else {
            return None;
        };
        self.advance(len);
        Some(op)
    }

    // ── Addition: + - ────────────────────────────────────────

    fn addition(&mut self) -> Result<Expr, String> {
        let mut left = self.multiplication()?;
        loop {
            self.skip_ws();
            let op = match self.peek_char() {
                Some('+') => BinaryOp::Add,
                Some('-') => {
                    // Disambiguate: `- ` is subtraction, but at certain positions
                    // it could be part of a keyword. Check it's not `--` comment.
                    BinaryOp::Sub
                }
                _ => break,
            };
            self.advance(1);
            self.skip_ws();
            let right = self.multiplication()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // ── Multiplication: * / % ────────────────────────────────

    fn multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            self.skip_ws();
            let op = match self.peek_char() {
                Some('*') => BinaryOp::Mul,
                Some('/') => BinaryOp::Div,
                Some('%') => BinaryOp::Mod,
                _ => break,
            };
            self.advance(1);
            self.skip_ws();
            let right = self.unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // ── Unary: - ─────────────────────────────────────────────

    fn unary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek_char() == Some('-') {
            // Check it's not a negative number that should be a literal
            self.advance(1);
            self.skip_ws();
            let expr = self.primary()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            })
        } else {
            self.primary()
        }
    }

    // ── Primary ──────────────────────────────────────────────

    fn primary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        match self.peek_char() {
            None => Err("unexpected end of input".to_string()),
            Some('\'') => self.string_literal(),
            Some('(') => self.paren_expr(),
            Some('`') => self.backtick_identifier_expr(),
            Some(c) if c.is_ascii_digit() => self.numeric_literal(),
            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.keyword_or_ident(),
            Some('*') => {
                self.advance(1);
                Ok(Expr::Wildcard)
            }
            Some(c) => Err(format!(
                "unexpected character '{c}' at position {}",
                self.pos
            )),
        }
    }

    fn keyword_or_ident(&mut self) -> Result<Expr, String> {
        // Check for keywords first
        if self.peek_keyword("NULL") {
            self.try_keyword("NULL");
            return Ok(Expr::Literal(Literal::Null));
        }
        if self.peek_keyword("TRUE") {
            self.try_keyword("TRUE");
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }
        if self.peek_keyword("FALSE") {
            self.try_keyword("FALSE");
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }
        if self.peek_keyword("CASE") {
            return self.case_expr();
        }
        if self.peek_keyword("CAST") {
            return self.cast_expr();
        }

        // Identifier — could be function call or column ref
        let name = self.identifier()?;
        self.skip_ws();

        // Function call
        if self.peek_char() == Some('(') {
            self.advance(1);
            self.skip_ws();

            // count(*)
            if self.peek_char() == Some('*') {
                self.advance(1);
                self.skip_ws();
                self.expect_char(')')?;
                return Ok(Expr::FunctionCall {
                    name,
                    args: vec![Expr::Wildcard],
                    distinct: false,
                });
            }

            // Empty args
            if self.peek_char() == Some(')') {
                self.advance(1);
                return Ok(Expr::FunctionCall {
                    name,
                    args: vec![],
                    distinct: false,
                });
            }

            // DISTINCT
            let distinct = self.try_keyword("DISTINCT");
            if distinct {
                self.skip_ws();
            }

            let args = self.expression_list()?;
            self.skip_ws();
            self.expect_char(')')?;

            return Ok(Expr::FunctionCall {
                name,
                args,
                distinct,
            });
        }

        // Qualified column: name.column
        if self.peek_char() == Some('.') {
            self.advance(1);
            let col = self.identifier()?;
            return Ok(Expr::ColumnRef {
                table: Some(name),
                column: col,
            });
        }

        // Simple column
        Ok(Expr::ColumnRef {
            table: None,
            column: name,
        })
    }

    // ── Literals ─────────────────────────────────────────────

    fn string_literal(&mut self) -> Result<Expr, String> {
        self.advance(1); // consume opening '
        let mut s = String::new();
        loop {
            match self.peek_char() {
                None => return Err("unterminated string literal".to_string()),
                Some('\'') => {
                    self.advance(1);
                    // Escaped quote ''
                    if self.peek_char() == Some('\'') {
                        s.push('\'');
                        self.advance(1);
                    } else {
                        break;
                    }
                }
                Some(c) => {
                    s.push(c);
                    self.advance(c.len_utf8());
                }
            }
        }
        Ok(Expr::Literal(Literal::String(s)))
    }

    fn numeric_literal(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        while self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
            self.advance(1);
        }
        if self.peek_char() == Some('.') {
            self.advance(1);
            if !self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
                return Err(format!(
                    "expected digit after decimal point at position {}",
                    self.pos
                ));
            }
            while self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
                self.advance(1);
            }
            let val: f64 = self.input[start..self.pos]
                .parse()
                .map_err(|e| format!("invalid float: {e}"))?;
            Ok(Expr::Literal(Literal::Float(val)))
        } else {
            let val: i64 = self.input[start..self.pos]
                .parse()
                .map_err(|e| format!("invalid integer: {e}"))?;
            Ok(Expr::Literal(Literal::Integer(val)))
        }
    }

    // ── CASE expression ──────────────────────────────────────

    fn case_expr(&mut self) -> Result<Expr, String> {
        self.try_keyword("CASE");
        self.skip_ws();

        let mut when_clauses = Vec::new();
        while self.try_keyword("WHEN") {
            self.skip_ws();
            let cond = self.expression()?;
            self.skip_ws();
            if !self.try_keyword("THEN") {
                return Err(format!("expected THEN at position {}", self.pos));
            }
            self.skip_ws();
            let then = self.expression()?;
            self.skip_ws();
            when_clauses.push((cond, then));
        }

        let else_clause = if self.try_keyword("ELSE") {
            self.skip_ws();
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.skip_ws();
        if !self.try_keyword("END") {
            return Err(format!("expected END at position {}", self.pos));
        }

        Ok(Expr::Case {
            when_clauses,
            else_clause,
        })
    }

    // ── CAST expression ──────────────────────────────────────

    fn cast_expr(&mut self) -> Result<Expr, String> {
        self.try_keyword("CAST");
        self.skip_ws();
        self.expect_char('(')?;
        self.skip_ws();
        // Parse the inner expression without allowing AS alias (or_expr level)
        let expr = self.or_expr()?;
        self.skip_ws();
        if !self.try_keyword("AS") {
            return Err(format!("expected AS in CAST at position {}", self.pos));
        }
        self.skip_ws();
        let data_type = self.type_name()?;
        self.skip_ws();
        self.expect_char(')')?;
        Ok(Expr::Cast {
            expr: Box::new(expr),
            data_type,
        })
    }

    fn type_name(&mut self) -> Result<String, String> {
        let mut s = String::new();
        let mut depth = 0i32;
        self.skip_ws();
        loop {
            match self.peek_char() {
                None => break,
                Some('(' | '<') => {
                    depth += 1;
                    s.push(self.peek_char().unwrap());
                    self.advance(1);
                }
                Some(c @ (')' | '>')) => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    s.push(c);
                    self.advance(1);
                }
                Some(',') if depth == 0 => break,
                Some(c) => {
                    s.push(c);
                    self.advance(c.len_utf8());
                }
            }
        }
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            return Err(format!("expected type name at position {}", self.pos));
        }
        Ok(trimmed)
    }

    // ── Parenthesized expression ─────────────────────────────

    fn paren_expr(&mut self) -> Result<Expr, String> {
        self.advance(1); // consume (
        self.skip_ws();
        let expr = self.expression()?;
        self.skip_ws();
        self.expect_char(')')?;
        Ok(expr)
    }

    // ── Backtick identifier ──────────────────────────────────

    fn backtick_identifier_expr(&mut self) -> Result<Expr, String> {
        let name = self.backtick_ident()?;
        Ok(Expr::ColumnRef {
            table: None,
            column: name,
        })
    }

    fn backtick_ident(&mut self) -> Result<String, String> {
        self.advance(1); // consume `
        let start = self.pos;
        while self.peek_char().is_some_and(|c| c != '`') {
            self.advance(1);
        }
        if self.peek_char() != Some('`') {
            return Err("unterminated backtick identifier".to_string());
        }
        let name = self.input[start..self.pos].to_string();
        self.advance(1); // consume closing `
        Ok(name)
    }

    // ── Identifiers ──────────────────────────────────────────

    fn identifier(&mut self) -> Result<String, String> {
        if self.peek_char() == Some('`') {
            return self.backtick_ident();
        }
        let start = self.pos;
        match self.peek_char() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.advance(1),
            _ => return Err(format!("expected identifier at position {}", self.pos)),
        }
        while self
            .peek_char()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.advance(1);
        }
        Ok(self.input[start..self.pos].to_string())
    }

    // ── Expression list ──────────────────────────────────────

    fn expression_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut exprs = vec![self.expression()?];
        loop {
            self.skip_ws();
            if self.peek_char() == Some(',') {
                self.advance(1);
                self.skip_ws();
                exprs.push(self.expression()?);
            } else {
                break;
            }
        }
        Ok(exprs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Expr {
        parse_expression(input).unwrap_or_else(|e| panic!("failed to parse '{input}': {e}"))
    }

    #[test]
    fn simple_column() {
        assert_eq!(
            parse("name"),
            Expr::ColumnRef {
                table: None,
                column: "name".into()
            }
        );
    }

    #[test]
    fn qualified_column() {
        assert_eq!(
            parse("employees.name"),
            Expr::ColumnRef {
                table: Some("employees".into()),
                column: "name".into()
            }
        );
    }

    #[test]
    fn integer_literal() {
        assert_eq!(parse("42"), Expr::Literal(Literal::Integer(42)));
    }

    #[test]
    fn float_literal() {
        assert_eq!(parse("3.15"), Expr::Literal(Literal::Float(3.15)));
    }

    #[test]
    fn string_literal_simple() {
        assert_eq!(
            parse("'hello'"),
            Expr::Literal(Literal::String("hello".into()))
        );
    }

    #[test]
    fn string_literal_escaped_quote() {
        assert_eq!(
            parse("'it''s'"),
            Expr::Literal(Literal::String("it's".into()))
        );
    }

    #[test]
    fn null_literal() {
        assert_eq!(parse("NULL"), Expr::Literal(Literal::Null));
        assert_eq!(parse("null"), Expr::Literal(Literal::Null));
    }

    #[test]
    fn boolean_literals() {
        assert_eq!(parse("true"), Expr::Literal(Literal::Boolean(true)));
        assert_eq!(parse("false"), Expr::Literal(Literal::Boolean(false)));
    }

    #[test]
    fn arithmetic() {
        let expr = parse("salary * 1.1");
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn arithmetic_precedence() {
        let expr = parse("a + b * c");
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Add,
                right,
                ..
            } => {
                assert!(matches!(
                    *right,
                    Expr::BinaryOp {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            }
            other => panic!("expected Add(_, Mul(_, _)), got {other:?}"),
        }
    }

    #[test]
    fn comparison_eq() {
        let expr = parse("status = 'active'");
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::Eq,
                ..
            }
        ));
    }

    #[test]
    fn comparison_double_eq() {
        let expr = parse("x == 1");
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::Eq,
                ..
            }
        ));
    }

    #[test]
    fn and_or_precedence() {
        // (a=1 AND b=2) OR c=3
        let expr = parse("a = 1 AND b = 2 OR c = 3");
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn not_expression() {
        let expr = parse("NOT active");
        assert!(matches!(
            expr,
            Expr::UnaryOp {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn is_null() {
        let expr = parse("x IS NULL");
        assert!(matches!(expr, Expr::IsNull { negated: false, .. }));
    }

    #[test]
    fn is_not_null() {
        let expr = parse("x IS NOT NULL");
        assert!(matches!(expr, Expr::IsNull { negated: true, .. }));
    }

    #[test]
    fn in_list() {
        let expr = parse("status IN ('a', 'b', 'c')");
        match expr {
            Expr::InList { list, negated, .. } => {
                assert!(!negated);
                assert_eq!(list.len(), 3);
            }
            other => panic!("expected InList, got {other:?}"),
        }
    }

    #[test]
    fn not_in_list() {
        let expr = parse("x NOT IN (1, 2)");
        assert!(matches!(expr, Expr::InList { negated: true, .. }));
    }

    #[test]
    fn between() {
        let expr = parse("age BETWEEN 18 AND 65");
        assert!(matches!(expr, Expr::Between { negated: false, .. }));
    }

    #[test]
    fn like_pattern() {
        let expr = parse("name LIKE '%smith%'");
        assert!(matches!(expr, Expr::Like { negated: false, .. }));
    }

    #[test]
    fn function_call() {
        match parse("upper(name)") {
            Expr::FunctionCall {
                name,
                args,
                distinct,
            } => {
                assert_eq!(name, "upper");
                assert_eq!(args.len(), 1);
                assert!(!distinct);
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn count_star() {
        match parse("count(*)") {
            Expr::FunctionCall { name, args, .. } => {
                assert_eq!(name, "count");
                assert_eq!(args, vec![Expr::Wildcard]);
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn count_distinct() {
        match parse("count(DISTINCT customer_id)") {
            Expr::FunctionCall { name, distinct, .. } => {
                assert_eq!(name, "count");
                assert!(distinct);
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn nested_functions() {
        match parse("upper(trim(name))") {
            Expr::FunctionCall { name, args, .. } => {
                assert_eq!(name, "upper");
                assert!(matches!(args[0], Expr::FunctionCall { .. }));
            }
            other => panic!("expected nested FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn case_expression() {
        match parse("CASE WHEN amount > 1000 THEN 'high' ELSE 'low' END") {
            Expr::Case {
                when_clauses,
                else_clause,
            } => {
                assert_eq!(when_clauses.len(), 1);
                assert!(else_clause.is_some());
            }
            other => panic!("expected Case, got {other:?}"),
        }
    }

    #[test]
    fn case_multi_when() {
        match parse(
            "CASE WHEN amount > 1000 THEN 'high' WHEN amount > 100 THEN 'medium' ELSE 'low' END",
        ) {
            Expr::Case { when_clauses, .. } => assert_eq!(when_clauses.len(), 2),
            other => panic!("expected Case, got {other:?}"),
        }
    }

    #[test]
    fn cast_expression() {
        match parse("CAST(amount AS double)") {
            Expr::Cast { data_type, .. } => assert_eq!(data_type, "double"),
            other => panic!("expected Cast, got {other:?}"),
        }
    }

    #[test]
    fn aliased_expression() {
        match parse("salary * 1.1 as adjusted_salary") {
            Expr::Alias { alias, .. } => assert_eq!(alias, "adjusted_salary"),
            other => panic!("expected Alias, got {other:?}"),
        }
    }

    #[test]
    fn aggregate_with_alias() {
        match parse("sum(amount) as total") {
            Expr::Alias { expr, alias } => {
                assert_eq!(alias, "total");
                assert!(matches!(*expr, Expr::FunctionCall { .. }));
            }
            other => panic!("expected Alias(FunctionCall), got {other:?}"),
        }
    }

    #[test]
    fn complex_filter() {
        let expr = parse("status = 'active' AND created_at >= '2025-01-01'");
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::And,
                ..
            }
        ));
    }

    #[test]
    fn backtick_identifier() {
        assert_eq!(
            parse("`my column`"),
            Expr::ColumnRef {
                table: None,
                column: "my column".into()
            }
        );
    }

    #[test]
    fn no_args_function() {
        match parse("current_timestamp()") {
            Expr::FunctionCall { name, args, .. } => {
                assert_eq!(name, "current_timestamp");
                assert!(args.is_empty());
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn negation() {
        assert!(matches!(
            parse("-amount"),
            Expr::UnaryOp {
                op: UnaryOp::Neg,
                ..
            }
        ));
    }

    #[test]
    fn parenthesized() {
        assert!(matches!(
            parse("(a + b) * c"),
            Expr::BinaryOp {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn multi_arg_function() {
        match parse("concat(first_name, ' ', last_name)") {
            Expr::FunctionCall { name, args, .. } => {
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 3);
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn join_condition_qualified() {
        match parse("employees.dept_id = departments.id") {
            Expr::BinaryOp {
                left,
                op: BinaryOp::Eq,
                right,
            } => {
                assert!(matches!(*left, Expr::ColumnRef { table: Some(_), .. }));
                assert!(matches!(*right, Expr::ColumnRef { table: Some(_), .. }));
            }
            other => panic!("expected Eq of qualified cols, got {other:?}"),
        }
    }

    #[test]
    fn lag_with_offset() {
        match parse("lag(salary, 1)") {
            Expr::FunctionCall { name, args, .. } => {
                assert_eq!(name, "lag");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected FunctionCall, got {other:?}"),
        }
    }

    #[test]
    fn empty_expression_fails() {
        assert!(parse_expression("").is_err());
        assert!(parse_expression("  ").is_err());
    }

    #[test]
    fn not_a_and_b_or_c() {
        // Per spec §9.2: NOT a AND b OR c → ((NOT a) AND b) OR c
        let expr = parse("NOT a AND b OR c");
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn count_1_as_alias() {
        match parse("count(1) as num_sales") {
            Expr::Alias { expr, alias } => {
                assert_eq!(alias, "num_sales");
                assert!(matches!(*expr, Expr::FunctionCall { .. }));
            }
            other => panic!("expected Alias, got {other:?}"),
        }
    }
}
