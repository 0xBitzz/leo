// Copyright (C) 2019-2023 Aleo Systems Inc.
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

use crate::Flattener;
use itertools::Itertools;

use leo_ast::{
    AccessExpression,
    ArrayAccess,
    AssociatedFunction,
    Expression,
    ExpressionReconstructor,
    Member,
    MemberAccess,
    Statement,
    StructExpression,
    StructVariableInitializer,
    TernaryExpression,
    TupleExpression,
};

// TODO: Clean up logic. To be done in a follow-up PR (feat/tuples)

impl ExpressionReconstructor for Flattener<'_> {
    type AdditionalOutput = Vec<Statement>;

    /// Replaces a tuple access expression with the appropriate expression.
    fn reconstruct_access(&mut self, input: AccessExpression) -> (Expression, Self::AdditionalOutput) {
        let mut statements = Vec::new();
        (
            match input {
                AccessExpression::Array(array) => Expression::Access(AccessExpression::Array(ArrayAccess {
                    array: Box::new(self.reconstruct_expression(*array.array).0),
                    index: Box::new(self.reconstruct_expression(*array.index).0),
                    span: array.span,
                    id: array.id,
                })),
                AccessExpression::AssociatedFunction(function) => {
                    Expression::Access(AccessExpression::AssociatedFunction(AssociatedFunction {
                        ty: function.ty,
                        name: function.name,
                        arguments: function
                            .arguments
                            .into_iter()
                            .map(|arg| self.reconstruct_expression(arg).0)
                            .collect(),
                        span: function.span,
                        id: function.id,
                    }))
                }
                AccessExpression::Member(member) => Expression::Access(AccessExpression::Member(MemberAccess {
                    inner: Box::new(self.reconstruct_expression(*member.inner).0),
                    name: member.name,
                    span: member.span,
                    id: member.id,
                })),
                AccessExpression::Tuple(tuple) => {
                    // Reconstruct the tuple expression.
                    let (expr, stmts) = self.reconstruct_expression(*tuple.tuple);

                    // Accumulate any statements produced.
                    statements.extend(stmts);

                    // Lookup the expression in the tuple map.
                    match expr {
                        Expression::Identifier(identifier) => {
                            // Note that this unwrap is safe since TYC guarantees that all tuples are declared and indices are valid.
                            self.tuples.get(&identifier.name).unwrap().elements[tuple.index.value()].clone()
                        }
                        _ => unreachable!("SSA guarantees that subexpressions are identifiers or literals."),
                    }
                }
                AccessExpression::AssociatedConstant(access) => {
                    Expression::Access(AccessExpression::AssociatedConstant(access))
                }
            },
            statements,
        )
    }

    /// Reconstructs a struct init expression, flattening any tuples in the expression.
    fn reconstruct_struct_init(&mut self, input: StructExpression) -> (Expression, Self::AdditionalOutput) {
        let mut statements = Vec::new();
        let mut members = Vec::with_capacity(input.members.len());

        // Reconstruct and flatten the argument expressions.
        for member in input.members.into_iter() {
            // Note that this unwrap is safe since SSA guarantees that all struct variable initializers are of the form `<name>: <expr>`.
            let (expr, stmts) = self.reconstruct_expression(member.expression.unwrap());
            // Accumulate any statements produced.
            statements.extend(stmts);
            // Accumulate the struct members.
            members.push(StructVariableInitializer {
                identifier: member.identifier,
                expression: Some(expr),
                span: member.span,
                id: member.id,
            });
        }

        (Expression::Struct(StructExpression { name: input.name, members, span: input.span, id: input.id }), statements)
    }

    /// Reconstructs ternary expressions over arrays, structs, and tuples, accumulating any statements that are generated.
    /// This is necessary because Aleo instructions does not support ternary expressions over composite data types.
    /// For example, the ternary expression `cond ? (a, b) : (c, d)` is flattened into the following:
    /// ```leo
    /// let var$0 = cond ? a : c;
    /// let var$1 = cond ? b : d;
    /// (var$0, var$1)
    /// ```
    /// For structs, the ternary expression `cond ? a : b`, where `a` and `b` are both structs `Foo { bar: u8, baz: u8 }`, is flattened into the following:
    /// ```leo
    /// let var$0 = cond ? a.bar : b.bar;
    /// let var$1 = cond ? a.baz : b.baz;
    /// let var$2 = Foo { bar: var$0, baz: var$1 };
    /// var$2
    /// ```
    fn reconstruct_ternary(&mut self, input: TernaryExpression) -> (Expression, Self::AdditionalOutput) {
        let mut statements = Vec::new();
        match (*input.if_true, *input.if_false) {
            // If both expressions are identifiers which are structs, construct ternary expression for each of the members and a struct expression for the result.
            (Expression::Identifier(first), Expression::Identifier(second))
                if self.structs.contains_key(&first.name) && self.structs.contains_key(&second.name) =>
            {
                let first_struct = self.symbol_table.lookup_struct(*self.structs.get(&first.name).unwrap()).unwrap();
                let second_struct = self.symbol_table.lookup_struct(*self.structs.get(&second.name).unwrap()).unwrap();
                // Note that type checking guarantees that both expressions have the same same type. This is a sanity check.
                assert_eq!(first_struct, second_struct);

                self.ternary_struct(first_struct, &input.condition, &first, &second)
            }
            // If both expressions are identifiers which map to tuples, construct ternary expression over the tuples.
            (Expression::Identifier(first), Expression::Identifier(second))
                if self.tuples.contains_key(&first.name) && self.tuples.contains_key(&second.name) =>
            {
                // Note that this unwrap is safe since we check that `self.tuples` contains the key.
                let first_tuple = self.tuples.get(&first.name).unwrap();
                // Note that this unwrap is safe since we check that `self.tuples` contains the key.
                let second_tuple = self.tuples.get(&second.name).unwrap();
                // Note that type checking guarantees that both expressions have the same same type.
                self.reconstruct_ternary(TernaryExpression {
                    condition: input.condition,
                    if_true: Box::new(Expression::Tuple(first_tuple.clone())),
                    if_false: Box::new(Expression::Tuple(second_tuple.clone())),
                    span: input.span,
                    id: input.id,
                })
            }
            // Otherwise, create a new intermediate assignment for the ternary expression are return the assigned variable.
            // Note that a new assignment must be created to flattened nested ternary expressions.
            (if_true, if_false) => {
                // Reconstruct the true case.
                let (if_true, stmts) = self.reconstruct_expression(if_true);
                statements.extend(stmts);

                // Reconstruct the false case.
                let (if_false, stmts) = self.reconstruct_expression(if_false);
                statements.extend(stmts);

                let (identifier, statement) =
                    self.unique_simple_assign_statement(Expression::Ternary(TernaryExpression {
                        condition: input.condition,
                        if_true: Box::new(if_true),
                        if_false: Box::new(if_false),
                        span: input.span,
                        id: input.id,
                    }));

                // Accumulate the new assignment statement.
                statements.push(statement);

                (Expression::Identifier(identifier), statements)
            }
        }
    }
}
