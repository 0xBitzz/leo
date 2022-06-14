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

use leo_ast::*;
use leo_errors::TypeCheckerError;

use crate::TypeChecker;

fn return_incorrect_type(t1: Option<Type>, t2: Option<Type>, expected: &Option<Type>) -> Option<Type> {
    match (t1, t2) {
        (Some(t1), Some(t2)) if t1 == t2 => Some(t1),
        (Some(t1), Some(t2)) => {
            if let Some(expected) = expected {
                if &t1 != expected {
                    Some(t1)
                } else {
                    Some(t2)
                }
            } else {
                Some(t1)
            }
        }
        (None, Some(_)) | (Some(_), None) | (None, None) => None,
    }
}

impl<'a> ExpressionVisitor<'a> for TypeChecker<'a> {
    type AdditionalInput = Option<Type>;
    type Output = Type;

    fn visit_expression(&mut self, input: &'a Expression, expected: &Self::AdditionalInput) -> Option<Self::Output> {
        return match input {
            Expression::Identifier(expr) => self.visit_identifier(expr, expected),
            Expression::Value(expr) => self.visit_value(expr, expected),
            Expression::Binary(expr) => self.visit_binary(expr, expected),
            Expression::Unary(expr) => self.visit_unary(expr, expected),
            Expression::Ternary(expr) => self.visit_ternary(expr, expected),
            Expression::Call(expr) => self.visit_call(expr, expected),
            Expression::Err(expr) => self.visit_err(expr, expected),
        };
    }

    fn visit_identifier(&mut self, input: &'a Identifier, expected: &Self::AdditionalInput) -> Option<Self::Output> {
        if let Some(var) = self.symbol_table.lookup_variable(input.name) {
            Some(self.assert_type(*var.type_, expected, var.span))
        } else {
            self.handler
                .emit_err(TypeCheckerError::unknown_sym("variable", input.name, input.span()).into());
            None
        }
    }

    fn visit_value(&mut self, input: &'a ValueExpression, expected: &Self::AdditionalInput) -> Option<Self::Output> {
        return Some(match input {
            ValueExpression::Address(_, _) => self.assert_type(Type::Address, expected, input.span()),
            ValueExpression::Boolean(_, _) => self.assert_type(Type::Boolean, expected, input.span()),
            ValueExpression::Field(_, _) => self.assert_type(Type::Field, expected, input.span()),
            ValueExpression::Integer(type_, str_content, _) => {
                match type_ {
                    IntegerType::I8 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if int.parse::<i8>().is_err() {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i8", input.span()).into());
                        }
                    }
                    IntegerType::I16 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if int.parse::<i16>().is_err() {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i16", input.span()).into());
                        }
                    }
                    IntegerType::I32 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if int.parse::<i32>().is_err() {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i32", input.span()).into());
                        }
                    }
                    IntegerType::I64 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if int.parse::<i64>().is_err() {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i64", input.span()).into());
                        }
                    }
                    IntegerType::I128 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if int.parse::<i128>().is_err() {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i128", input.span()).into());
                        }
                    }
                    IntegerType::U8 if str_content.parse::<u8>().is_err() => self
                        .handler
                        .emit_err(TypeCheckerError::invalid_int_value(str_content, "u8", input.span()).into()),
                    IntegerType::U16 if str_content.parse::<u16>().is_err() => self
                        .handler
                        .emit_err(TypeCheckerError::invalid_int_value(str_content, "u16", input.span()).into()),
                    IntegerType::U32 if str_content.parse::<u32>().is_err() => self
                        .handler
                        .emit_err(TypeCheckerError::invalid_int_value(str_content, "u32", input.span()).into()),
                    IntegerType::U64 if str_content.parse::<u64>().is_err() => self
                        .handler
                        .emit_err(TypeCheckerError::invalid_int_value(str_content, "u64", input.span()).into()),
                    IntegerType::U128 if str_content.parse::<u128>().is_err() => self
                        .handler
                        .emit_err(TypeCheckerError::invalid_int_value(str_content, "u128", input.span()).into()),
                    _ => {}
                }
                self.assert_type(Type::IntegerType(*type_), expected, input.span())
            }
            ValueExpression::Group(_) => self.assert_type(Type::Group, expected, input.span()),
            ValueExpression::Scalar(_, _) => self.assert_type(Type::Scalar, expected, input.span()),
            ValueExpression::String(_, _) => self.assert_type(Type::String, expected, input.span()),
        });
    }

    fn visit_binary(&mut self, input: &'a BinaryExpression, expected: &Self::AdditionalInput) -> Option<Self::Output> {
        match input.op {
            BinaryOperation::And | BinaryOperation::Or => {
                self.assert_type(Type::Boolean, expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                return_incorrect_type(t1, t2, expected)
            }
            BinaryOperation::Add => {
                self.assert_field_group_scalar_int_type(expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                return_incorrect_type(t1, t2, expected)
            }
            BinaryOperation::Sub => {
                self.assert_field_group_int_type(expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                return_incorrect_type(t1, t2, expected)
            }
            BinaryOperation::Mul => {
                self.assert_field_group_int_type(expected, input.span());

                let t1 = self.visit_expression(&input.left, &None);
                let t2 = self.visit_expression(&input.right, &None);

                // Allow `group` * `scalar` multiplication.
                match (t1.as_ref(), t2.as_ref()) {
                    (Some(Type::Group), Some(other)) => {
                        self.assert_type(Type::Group, expected, input.left.span());
                        self.assert_type(*other, &Some(Type::Scalar), input.right.span());
                        Some(Type::Group)
                    }
                    (Some(other), Some(Type::Group)) => {
                        self.assert_type(*other, &Some(Type::Scalar), input.left.span());
                        self.assert_type(Type::Group, expected, input.right.span());
                        Some(Type::Group)
                    }
                    (Some(t1), Some(t2)) => {
                        self.assert_type(*t1, expected, input.left.span());
                        self.assert_type(*t2, expected, input.right.span());
                        return_incorrect_type(Some(*t1), Some(*t2), expected)
                    }
                    (Some(type_), None) => {
                        self.assert_type(*type_, expected, input.left.span());
                        None
                    }
                    (None, Some(type_)) => {
                        self.assert_type(*type_, expected, input.right.span());
                        None
                    }
                    (None, None) => None,
                }
            }
            BinaryOperation::Div => {
                self.assert_field_int_type(expected, input.span());

                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                return_incorrect_type(t1, t2, expected)
            }
            BinaryOperation::Pow => {
                let t1 = self.visit_expression(&input.left, &None);
                let t2 = self.visit_expression(&input.right, &None);

                match (t1.as_ref(), t2.as_ref()) {
                    // Type A must be an int.
                    // Type B must be a unsigned int.
                    (Some(Type::IntegerType(_)), Some(Type::IntegerType(itype))) if !itype.is_signed() => {
                        self.assert_type(t1.unwrap(), expected, input.left.span());
                    }
                    // Type A was an int.
                    // But Type B was not a unsigned int.
                    (Some(Type::IntegerType(_)), Some(t)) => {
                        self.handler.emit_err(
                            TypeCheckerError::incorrect_pow_exponent_type("unsigned int", t, input.right.span()).into(),
                        );
                    }
                    // Type A must be a field.
                    // Type B must be an int.
                    (Some(Type::Field), Some(Type::IntegerType(_))) => {
                        self.assert_type(Type::Field, expected, input.left.span());
                    }
                    // Type A was a field.
                    // But Type B was not an int.
                    (Some(Type::Field), Some(t)) => {
                        self.handler.emit_err(
                            TypeCheckerError::incorrect_pow_exponent_type("int", t, input.right.span()).into(),
                        );
                    }
                    // The base is some type thats not an int or field.
                    (Some(t), _) if !matches!(t, Type::IntegerType(_) | Type::Field) => {
                        self.handler
                            .emit_err(TypeCheckerError::incorrect_pow_base_type(t, input.left.span()).into());
                    }
                    _ => {}
                }

                t1
            }
            BinaryOperation::Eq | BinaryOperation::Ne => {
                let t1 = self.visit_expression(&input.left, &None);
                let t2 = self.visit_expression(&input.right, &None);

                self.assert_eq_types(t1, t2, input.span());

                Some(Type::Boolean)
            }
            BinaryOperation::Lt | BinaryOperation::Gt | BinaryOperation::Le | BinaryOperation::Ge => {
                let t1 = self.visit_expression(&input.left, &None);
                self.assert_field_scalar_int_type(&t1, input.left.span());

                let t2 = self.visit_expression(&input.right, &None);
                self.assert_field_scalar_int_type(&t2, input.right.span());

                self.assert_eq_types(t1, t2, input.span());

                Some(Type::Boolean)
            }
        }
    }

    fn visit_unary(&mut self, input: &'a UnaryExpression, expected: &Self::AdditionalInput) -> Option<Self::Output> {
        match input.op {
            UnaryOperation::Not => {
                self.assert_type(Type::Boolean, expected, input.span());
                self.visit_expression(&input.inner, expected)
            }
            UnaryOperation::Negate => {
                let prior_negate_state = self.negate;
                self.negate = true;

                let type_ = self.visit_expression(&input.inner, expected);
                self.negate = prior_negate_state;
                match type_.as_ref() {
                    Some(
                        Type::IntegerType(
                            IntegerType::I8
                            | IntegerType::I16
                            | IntegerType::I32
                            | IntegerType::I64
                            | IntegerType::I128,
                        )
                        | Type::Field
                        | Type::Group,
                    ) => {}
                    Some(t) => self
                        .handler
                        .emit_err(TypeCheckerError::type_is_not_negatable(t, input.inner.span()).into()),
                    _ => {}
                };
                type_
            }
        }
    }

    fn visit_ternary(
        &mut self,
        input: &'a TernaryExpression,
        expected: &Self::AdditionalInput,
    ) -> Option<Self::Output> {
        self.visit_expression(&input.condition, &Some(Type::Boolean));

        let t1 = self.visit_expression(&input.if_true, expected);
        let t2 = self.visit_expression(&input.if_false, expected);

        return_incorrect_type(t1, t2, expected)
    }

    fn visit_call(&mut self, input: &'a CallExpression, expected: &Self::AdditionalInput) -> Option<Self::Output> {
        match &*input.function {
            Expression::Identifier(ident) => {
                if let Some(func) = self.symbol_table.lookup_fn(ident.name) {
                    let ret = self.assert_type(*func.type_, expected, func.span);

                    if func.input.len() != input.arguments.len() {
                        self.handler.emit_err(
                            TypeCheckerError::incorrect_num_args_to_call(
                                func.input.len(),
                                input.arguments.len(),
                                input.span(),
                            )
                            .into(),
                        );
                    }

                    func.input
                        .iter()
                        .zip(input.arguments.iter())
                        .for_each(|(expected, argument)| {
                            self.visit_expression(argument, &Some(expected.get_variable().type_));
                        });

                    Some(ret)
                } else {
                    self.handler
                        .emit_err(TypeCheckerError::unknown_sym("function", &ident.name, ident.span()).into());
                    None
                }
            }
            expr => self.visit_expression(expr, expected),
        }
    }
}
