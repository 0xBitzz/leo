// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

use leo_errors::{ParserError, Result};

const INT_TYPES: &[Token] = &[
    Token::I8,
    Token::I16,
    Token::I32,
    Token::I64,
    Token::I128,
    Token::U8,
    Token::U16,
    Token::U32,
    Token::U64,
    Token::U128,
    Token::Field,
    Token::Group,
    Token::Scalar,
];

impl ParserContext<'_> {
    /// Returns an [`Expression`] AST node if the next token is an expression.
    /// Includes circuit init expressions.
    pub(crate) fn parse_expression(&mut self) -> Result<Expression> {
        // Store current parser state.
        let prior_fuzzy_state = self.disallow_circuit_construction;

        // Allow circuit init expressions.
        self.disallow_circuit_construction = false;

        // Parse expression.
        let result = self.parse_conditional_expression();

        // Restore prior parser state.
        self.disallow_circuit_construction = prior_fuzzy_state;

        result
    }

    /// Returns an [`Expression`] AST node if the next tokens represent
    /// a ternary expression. May or may not include circuit init expressions.
    ///
    /// Otherwise, tries to parse the next token using [`parse_disjunctive_expression`].
    pub(super) fn parse_conditional_expression(&mut self) -> Result<Expression> {
        // Try to parse the next expression. Try BinaryOperation::Or.
        let mut expr = self.parse_disjunctive_expression()?;

        // Parse the rest of the ternary expression.
        if self.eat(&Token::Question) {
            let if_true = self.parse_expression()?;
            self.expect(&Token::Colon)?;
            let if_false = self.parse_conditional_expression()?;
            expr = Expression::Ternary(TernaryExpression {
                span: expr.span() + if_false.span(),
                condition: Box::new(expr),
                if_true: Box::new(if_true),
                if_false: Box::new(if_false),
            });
        }
        Ok(expr)
    }

    /// Constructs a binary expression `left op right`.
    fn bin_expr(left: Expression, right: Expression, op: BinaryOperation) -> Expression {
        Expression::Binary(BinaryExpression {
            span: left.span() + right.span(),
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// Parses a left-associative binary expression `<left> token <right>` using `f` for left/right.
    /// The `token` is translated to `op` in the AST.
    fn parse_bin_expr(
        &mut self,
        tokens: &[Token],
        mut f: impl FnMut(&mut Self) -> Result<Expression>,
    ) -> Result<Expression> {
        let mut expr = f(self)?;
        while let Some(op) = self.eat_bin_op(tokens) {
            expr = Self::bin_expr(expr, f(self)?, op);
        }
        Ok(expr)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent
    /// a binary or expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_conjunctive_expression`].
    fn parse_disjunctive_expression(&mut self) -> Result<Expression> {
        self.parse_bin_expr(&[Token::Or], Self::parse_conjunctive_expression)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// binary and expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_equality_expression`].
    fn parse_conjunctive_expression(&mut self) -> Result<Expression> {
        self.parse_bin_expr(&[Token::And], Self::parse_equality_expression)
    }

    /// Eats one of binary operators matching any in `tokens`.
    fn eat_bin_op(&mut self, tokens: &[Token]) -> Option<BinaryOperation> {
        self.eat_any(tokens).then(|| match &self.prev_token.token {
            Token::Eq => BinaryOperation::Eq,
            Token::NotEq => BinaryOperation::Ne,
            Token::Lt => BinaryOperation::Lt,
            Token::LtEq => BinaryOperation::Le,
            Token::Gt => BinaryOperation::Gt,
            Token::GtEq => BinaryOperation::Ge,
            Token::Add => BinaryOperation::Add,
            Token::Minus => BinaryOperation::Sub,
            Token::Mul => BinaryOperation::Mul,
            Token::Div => BinaryOperation::Div,
            Token::Or => BinaryOperation::Or,
            Token::And => BinaryOperation::And,
            Token::Exp => BinaryOperation::Pow,
            _ => unreachable!("`eat_bin_op` shouldn't produce this"),
        })
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// binary equals or not equals expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_ordering_expression`].
    fn parse_equality_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_ordering_expression()?;
        if let Some(op) = self.eat_bin_op(&[Token::Eq, Token::NotEq]) {
            let right = self.parse_ordering_expression()?;
            expr = Self::bin_expr(expr, right, op);
        }
        Ok(expr)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// binary relational expression: less than, less than or equals, greater than, greater than or equals.
    ///
    /// Otherwise, tries to parse the next token using [`parse_additive_expression`].
    fn parse_ordering_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_additive_expression()?;
        if let Some(op) = self.eat_bin_op(&[Token::Lt, Token::LtEq, Token::Gt, Token::GtEq]) {
            let right = self.parse_additive_expression()?;
            expr = Self::bin_expr(expr, right, op);
        }
        Ok(expr)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// binary addition or subtraction expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_mul_div_pow_expression`].
    fn parse_additive_expression(&mut self) -> Result<Expression> {
        self.parse_bin_expr(&[Token::Add, Token::Minus], Self::parse_multiplicative_expression)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// binary multiplication, division, or modulus expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_exponential_expression`].
    fn parse_multiplicative_expression(&mut self) -> Result<Expression> {
        self.parse_bin_expr(&[Token::Mul, Token::Div], Self::parse_exponential_expression)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// binary exponentiation expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_unary_expression`].
    fn parse_exponential_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary_expression()?;

        if let Some(op) = self.eat_bin_op(&[Token::Exp]) {
            let right = self.parse_exponential_expression()?;
            expr = Self::bin_expr(expr, right, op);
        }

        Ok(expr)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// unary not, negate, or bitwise not expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_postfix_expression`].
    pub(super) fn parse_unary_expression(&mut self) -> Result<Expression> {
        let mut ops = Vec::new();
        while self.eat_any(&[Token::Not, Token::Minus]) {
            let operation = match self.prev_token.token {
                Token::Not => UnaryOperation::Not,
                Token::Minus => UnaryOperation::Negate,
                _ => unreachable!("parse_unary_expression_ shouldn't produce this"),
            };
            ops.push((operation, self.prev_token.span));
        }
        let mut inner = self.parse_postfix_expression()?;
        for (op, op_span) in ops.into_iter().rev() {
            inner = Expression::Unary(UnaryExpression {
                span: op_span + inner.span(),
                op,
                inner: Box::new(inner),
            });
        }
        Ok(inner)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent an
    /// array access, circuit member access, function call, or static function call expression.
    ///
    /// Otherwise, tries to parse the next token using [`parse_primary_expression`].
    fn parse_postfix_expression(&mut self) -> Result<Expression> {
        // We don't directly parse named-type's and Identifier's here as
        // the ABNF states. Rather the primary expression already
        // handle those. The ABNF is more specific for language reasons.
        let mut expr = self.parse_primary_expression()?;
        loop {
            if self.eat(&Token::Dot) {
                let curr = &self.token;
                return Err(ParserError::unexpected_str(&curr.token, "int or ident", curr.span).into());
            }

            if !self.check(&Token::LeftParen) {
                break;
            }

            let (arguments, _, span) = self.parse_paren_comma_list(|p| p.parse_expression().map(Some))?;
            expr = Expression::Call(CallExpression {
                span: expr.span() + span,
                function: Box::new(expr),
                arguments,
            });
        }
        Ok(expr)
    }

    /// Returns an [`Expression`] AST node if the next tokens represent a
    /// tuple initialization expression or an affine group literal.
    fn parse_tuple_expression(&mut self) -> Result<Expression> {
        if let Some(gt) = self.eat_group_partial().transpose()? {
            return Ok(Expression::Value(ValueExpression::Group(Box::new(GroupValue::Tuple(
                gt,
            )))));
        }

        let (mut tuple, trailing, span) = self.parse_paren_comma_list(|p| p.parse_expression().map(Some))?;

        if !trailing && tuple.len() == 1 {
            Ok(tuple.remove(0))
        } else {
            Err(ParserError::unexpected("A tuple expression.", "A valid expression.", span).into())
        }
    }

    /// Returns a reference to the next token if it is a [`GroupCoordinate`], or [None] if
    /// the next token is not a [`GroupCoordinate`].
    fn peek_group_coordinate(&self, dist: &mut usize) -> Option<GroupCoordinate> {
        let (advanced, gc) = self.look_ahead(*dist, |t0| match &t0.token {
            Token::Add => Some((1, GroupCoordinate::SignHigh)),
            Token::Minus => self.look_ahead(*dist + 1, |t1| match &t1.token {
                Token::Int(value) => Some((2, GroupCoordinate::Number(format!("-{}", value), t1.span))),
                _ => Some((1, GroupCoordinate::SignLow)),
            }),
            Token::Underscore => Some((1, GroupCoordinate::Inferred)),
            Token::Int(value) => Some((1, GroupCoordinate::Number(value.clone(), t0.span))),
            _ => None,
        })?;
        *dist += advanced;
        Some(gc)
    }

    /// Removes the next two tokens if they are a pair of [`GroupCoordinate`] and returns them,
    /// or [None] if the next token is not a [`GroupCoordinate`].
    fn eat_group_partial(&mut self) -> Option<Result<GroupTuple>> {
        assert!(self.check(&Token::LeftParen)); // `(`.

        // Peek at first gc.
        let start_span = &self.token.span;
        let mut dist = 1; // 0th is `(` so 1st is first gc's start.
        let first_gc = self.peek_group_coordinate(&mut dist)?;

        let check_ahead = |d, token: &_| self.look_ahead(d, |t| (&t.token == token).then(|| t.span));

        // Peek at `,`.
        check_ahead(dist, &Token::Comma)?;
        dist += 1; // Standing at `,` so advance one for next gc's start.

        // Peek at second gc.
        let second_gc = self.peek_group_coordinate(&mut dist)?;

        // Peek at `)`.
        let right_paren_span = check_ahead(dist, &Token::RightParen)?;
        dist += 1; // Standing at `)` so advance one for 'group'.

        // Peek at `group`.
        let end_span = check_ahead(dist, &Token::Group)?;
        dist += 1; // Standing at `)` so advance one for 'group'.

        let gt = GroupTuple {
            span: start_span + &end_span,
            x: first_gc,
            y: second_gc,
        };

        // Eat everything so that this isn't just peeking.
        for _ in 0..dist {
            self.bump();
        }

        if let Err(e) = assert_no_whitespace(right_paren_span, end_span, &format!("({},{})", gt.x, gt.y), "group") {
            return Some(Err(e));
        }

        Some(Ok(gt))
    }

    /// Returns an [`Expression`] AST node if the next token is a primary expression:
    /// - Literals: field, group, unsigned integer, signed integer, boolean, address
    /// - Aggregate types: array, tuple
    /// - Identifiers: variables, keywords
    /// - self
    ///
    /// Returns an expression error if the token cannot be matched.
    fn parse_primary_expression(&mut self) -> Result<Expression> {
        if let Token::LeftParen = self.token.token {
            return self.parse_tuple_expression();
        }

        let SpannedToken { token, span } = self.token.clone();
        self.bump();

        Ok(match token {
            Token::Int(value) => {
                let suffix_span = self.token.span;
                let full_span = span + suffix_span;
                let assert_no_whitespace = |x| assert_no_whitespace(span, suffix_span, &value, x);
                match self.eat_any(INT_TYPES).then(|| &self.prev_token.token) {
                    // Literal followed by `field`, e.g., `42field`.
                    Some(Token::Field) => {
                        assert_no_whitespace("field")?;
                        Expression::Value(ValueExpression::Field(value, full_span))
                    }
                    // Literal followed by `group`, e.g., `42group`.
                    Some(Token::Group) => {
                        assert_no_whitespace("group")?;
                        Expression::Value(ValueExpression::Group(Box::new(GroupValue::Single(value, full_span))))
                    }
                    // Literal followed by `scalar` e.g., `42scalar`.
                    Some(Token::Scalar) => {
                        assert_no_whitespace("scalar")?;
                        Expression::Value(ValueExpression::Scalar(value, full_span))
                    }
                    // Literal followed by other type suffix, e.g., `42u8`.
                    Some(suffix) => {
                        assert_no_whitespace(&suffix.to_string())?;
                        let int_ty = Self::token_to_int_type(suffix).expect("unknown int type token");
                        Expression::Value(ValueExpression::Integer(int_ty, value, full_span))
                    }
                    None => return Err(ParserError::implicit_values_not_allowed(value, span).into()),
                }
            }
            Token::True => Expression::Value(ValueExpression::Boolean(true, span)),
            Token::False => Expression::Value(ValueExpression::Boolean(false, span)),
            Token::AddressLit(value) => Expression::Value(ValueExpression::Address(value, span)),
            Token::StaticString(value) => Expression::Value(ValueExpression::String(value, span)),
            Token::Ident(name) => {
                let ident = Identifier { name, span };
                Expression::Identifier(ident)
            }
            t if crate::type_::TYPE_TOKENS.contains(&t) => Expression::Identifier(Identifier {
                name: t.keyword_to_symbol().unwrap(),
                span,
            }),
            token => {
                return Err(ParserError::unexpected_str(token, "expression", span).into());
            }
        })
    }
}

fn assert_no_whitespace(left_span: Span, right_span: Span, left: &str, right: &str) -> Result<()> {
    if left_span.hi != right_span.lo {
        let error_span = Span::new(left_span.hi, right_span.lo); // The span between them.
        return Err(ParserError::unexpected_whitespace(left, right, error_span).into());
    }

    Ok(())
}
