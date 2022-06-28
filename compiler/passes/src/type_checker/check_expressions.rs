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

use indexmap::IndexMap;
use leo_ast::*;
use leo_errors::TypeCheckerError;

use crate::{TypeChecker, Value};

use super::type_output::TypeOutput;

impl<'a> ExpressionVisitor<'a> for TypeChecker<'a> {
    type AdditionalInput = Option<Type>;
    type Output = TypeOutput;

    fn visit_expression(&mut self, input: &'a Expression, expected: &Self::AdditionalInput) -> Self::Output {
        match input {
            Expression::Access(expr) => self.visit_access(expr, expected),
            Expression::Identifier(expr) => self.visit_identifier(expr, expected),
            Expression::Literal(expr) => self.visit_literal(expr, expected),
            Expression::Binary(expr) => self.visit_binary(expr, expected),
            Expression::Unary(expr) => self.visit_unary(expr, expected),
            Expression::Ternary(expr) => self.visit_ternary(expr, expected),
            Expression::Call(expr) => self.visit_call(expr, expected),
            Expression::Err(expr) => self.visit_err(expr, expected),
            Expression::CircuitInit(expr) => self.visit_circuit_init(expr, expected),
        }
    }

    fn visit_identifier(&mut self, input: &'a Identifier, expected: &Self::AdditionalInput) -> Self::Output {
        if let Some(circuit) = self.symbol_table.borrow().lookup_circuit(&input.name) {
            self.assert_expected_option(
                Type::Identifier(circuit.identifier),
                TypeOutput::MutType(Type::Identifier(circuit.identifier)),
                expected,
                input.span(),
            )
        } else if let Some(var) = self.symbol_table.borrow().lookup_variable(&input.name) {
            self.assert_expected_option(var.type_, var, expected, input.span)
        } else {
            self.handler
                .emit_err(TypeCheckerError::unknown_sym("variable", input.name, input.span()));
            TypeOutput::None
        }
    }

    fn visit_literal(&mut self, input: &'a LiteralExpression, expected: &Self::AdditionalInput) -> Self::Output {
        match input {
            LiteralExpression::Address(value, span) => self.assert_expected_option(
                Type::Address,
                Value::Address(value.clone(), *span),
                expected,
                input.span(),
            ),
            LiteralExpression::Boolean(value, span) => {
                self.assert_expected_option(Type::Boolean, Value::Boolean(*value, *span), expected, input.span())
            }
            LiteralExpression::Circuit(_, _) => unreachable!("Circuits instantiations are not parsed as literals"),
            LiteralExpression::Field(value, span) => {
                self.assert_expected_option(Type::Field, Value::Field(value.clone(), *span), expected, input.span())
            }
            LiteralExpression::Integer(type_, str_content, _) => {
                let ret_type =
                    self.assert_expected_option(Type::IntegerType(*type_), TypeOutput::None, expected, input.span());
                match type_ {
                    IntegerType::I8 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if let Ok(int) = int.parse::<i8>() {
                            Value::I8(int, input.span()).into()
                        } else {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i8", input.span()));
                            ret_type
                        }
                    }
                    IntegerType::I16 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if let Ok(int) = int.parse::<i16>() {
                            Value::I16(int, input.span()).into()
                        } else {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i16", input.span()));
                            ret_type
                        }
                    }
                    IntegerType::I32 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if let Ok(int) = int.parse::<i32>() {
                            Value::I32(int, input.span()).into()
                        } else {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i32", input.span()));
                            ret_type
                        }
                    }
                    IntegerType::I64 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if let Ok(int) = int.parse::<i64>() {
                            Value::I64(int, input.span()).into()
                        } else {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i64", input.span()));
                            ret_type
                        }
                    }
                    IntegerType::I128 => {
                        let int = if self.negate {
                            format!("-{str_content}")
                        } else {
                            str_content.clone()
                        };

                        if let Ok(int) = int.parse::<i128>() {
                            Value::I128(int, input.span()).into()
                        } else {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(int, "i128", input.span()));
                            ret_type
                        }
                    }
                    IntegerType::U8 => match str_content.parse::<u8>() {
                        Ok(int) => Value::U8(int, input.span()).into(),
                        Err(_) => {
                            self.handler
                                .emit_err(TypeCheckerError::invalid_int_value(str_content, "u8", input.span()));
                            ret_type
                        }
                    },
                    IntegerType::U16 => match str_content.parse::<u16>() {
                        Ok(int) => Value::U16(int, input.span()).into(),
                        Err(_) => {
                            self.handler.emit_err(TypeCheckerError::invalid_int_value(
                                str_content,
                                "u16",
                                input.span(),
                            ));
                            ret_type
                        }
                    },
                    IntegerType::U32 => match str_content.parse::<u32>() {
                        Ok(int) => Value::U32(int, input.span()).into(),
                        Err(_) => {
                            self.handler.emit_err(TypeCheckerError::invalid_int_value(
                                str_content,
                                "u32",
                                input.span(),
                            ));
                            ret_type
                        }
                    },
                    IntegerType::U64 => match str_content.parse::<u64>() {
                        Ok(int) => Value::U64(int, input.span()).into(),
                        Err(_) => {
                            self.handler.emit_err(TypeCheckerError::invalid_int_value(
                                str_content,
                                "u64",
                                input.span(),
                            ));
                            ret_type
                        }
                    },
                    IntegerType::U128 => match str_content.parse::<u128>() {
                        Ok(int) => Value::U128(int, input.span()).into(),
                        Err(_) => {
                            self.handler.emit_err(TypeCheckerError::invalid_int_value(
                                str_content,
                                "u128",
                                input.span(),
                            ));
                            ret_type
                        }
                    },
                }
            }
            LiteralExpression::Group(value) => {
                self.assert_expected_option(Type::Group, Value::Group(value.clone()), expected, input.span())
            }
            LiteralExpression::Scalar(value, span) => self.assert_expected_option(
                Type::Scalar,
                Value::Scalar(value.clone(), *span),
                expected,
                input.span(),
            ),
            LiteralExpression::String(value, span) => self.assert_expected_option(
                Type::String,
                Value::String(value.clone(), *span),
                expected,
                input.span(),
            ),
        }
    }

    fn visit_access(&mut self, input: &'a AccessExpression, expected: &Self::AdditionalInput) -> Self::Output {
        // CAUTION: This implementation only allows access to core circuits.
        match input {
            AccessExpression::AssociatedFunction(access) => {
                // Check core circuit name and function.
                if let Some(core_instruction) = self.assert_core_circuit_call(&access.ty, &access.name) {
                    // Check num input arguments.
                    if core_instruction.num_args() != access.args.len() {
                        self.handler.emit_err(TypeCheckerError::incorrect_num_args_to_call(
                            core_instruction.num_args(),
                            access.args.len(),
                            input.span(),
                        ));
                    }

                    // Check first argument type.
                    if let Some(first_arg) = access.args.get(0usize) {
                        let first_arg_type = self.visit_expression(first_arg, &None);
                        self.assert_one_of_types(
                            &first_arg_type.into(),
                            core_instruction.first_arg_types(),
                            access.span(),
                        );
                    }

                    // Check second argument type.
                    if let Some(second_arg) = access.args.get(1usize) {
                        let second_arg_type = self.visit_expression(second_arg, &None);
                        self.assert_one_of_types(
                            &second_arg_type.into(),
                            core_instruction.second_arg_types(),
                            access.span(),
                        );
                    }

                    // Check return type.
                    self.assert_expected_option(
                        core_instruction.return_type(),
                        TypeOutput::MutType(core_instruction.return_type()),
                        expected,
                        access.span(),
                    )
                } else {
                    self.handler
                        .emit_err(TypeCheckerError::invalid_access_expression(access, access.span()));
                    TypeOutput::None
                }
            }
            AccessExpression::Member(access) => {
                let const_circuit = self.visit_expression(&access.inner, &None);
                let const_circuit_type = const_circuit.as_ref().into();
                let const_circuit_value: Option<Value> = const_circuit.as_ref().into();
                if let Some(Value::Circuit(_, const_members)) = const_circuit_value {
                    if let Some(const_member) = const_members.get(&access.name.name) {
                        const_circuit.replace_value(const_member.clone())
                    } else {
                        todo!("throw an error for member not existing");
                        TypeOutput::None
                    }
                } else if let Some(type_) = const_circuit_type {
                    if let Type::Identifier(ident) = type_ {
                        if let Some(circuit) = self.symbol_table.borrow().lookup_circuit(&ident.name) {
                            match circuit.members.get(&access.name.name) {
                                Some(CircuitMember::CircuitVariable(_, type_)) => const_circuit.replace(*type_),
                                None => {
                                    todo!("throw an error for member not existing");
                                    TypeOutput::None
                                }
                            }
                        } else {
                            todo!("circuit type does not exist");
                            TypeOutput::None
                        }
                    } else {
                        todo!("throw error non circuit type");
                        TypeOutput::None
                    }
                } else {
                    todo!("throw error here trying to access on a non circuit type");
                    TypeOutput::None
                }
            }
            _expr => TypeOutput::None, // todo: Add support for associated constants (u8::MAX).
        }
    }

    fn visit_binary(&mut self, input: &'a BinaryExpression, expected: &Self::AdditionalInput) -> Self::Output {
        match input.op {
            BinaryOperation::And | BinaryOperation::Or | BinaryOperation::Nand | BinaryOperation::Nor => {
                self.assert_expected_option(Type::Boolean, TypeOutput::None, expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                t1.return_incorrect_type(&t2, expected)
            }
            BinaryOperation::BitwiseAnd | BinaryOperation::BitwiseOr | BinaryOperation::Xor => {
                // Assert equal boolean or integer types.
                self.assert_bool_int_type(expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                t1.return_incorrect_type(&t2, expected)
            }
            BinaryOperation::Add => {
                self.assert_field_group_scalar_int_type(expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                t1.return_incorrect_type(&t2, expected)
            }
            BinaryOperation::Sub => {
                self.assert_field_group_int_type(expected, input.span());
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                t1.return_incorrect_type(&t2, expected)
            }
            BinaryOperation::Mul => {
                self.assert_field_group_int_type(expected, input.span());

                let t1 = self.visit_expression(&input.left, &None);
                let t2 = self.visit_expression(&input.right, &None);
                let combined = t1.return_incorrect_type(&t2, expected);

                // Allow `group` * `scalar` multiplication.
                match (t1.as_ref().into(), t2.as_ref().into()) {
                    (Some(Type::Group), other) => {
                        self.assert_expected_type(&other, TypeOutput::None, Type::Scalar, input.right.span());
                        self.assert_expected_type(expected, combined, Type::Group, input.span())
                    }
                    (other, Some(Type::Group)) => {
                        self.assert_expected_type(&other, TypeOutput::None, Type::Scalar, input.left.span());
                        self.assert_expected_type(expected, combined, Type::Group, input.span())
                    }
                    (_, _) => {
                        // Assert equal field or integer types.
                        self.assert_field_int_type(expected, input.span());

                        t1.return_incorrect_type(&t2, expected)
                    }
                }
            }
            BinaryOperation::Div => {
                self.assert_field_int_type(expected, input.span());

                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                t1.return_incorrect_type(&t2, expected)
            }
            BinaryOperation::Pow => {
                let t1 = self.visit_expression(&input.left, &None);
                let t2 = self.visit_expression(&input.right, &None);
                let combined = t1.return_incorrect_type(&t2, expected);

                match (t1.into(), t2.as_ref().into()) {
                    (Some(Type::Field), type_) => {
                        self.assert_expected_type(&type_, TypeOutput::None, Type::Field, input.right.span());
                        self.assert_expected_type(expected, combined, Type::Field, input.span())
                    }
                    (type_, Some(Type::Field)) => {
                        self.assert_expected_type(&type_, TypeOutput::None, Type::Field, input.left.span());
                        self.assert_expected_type(expected, combined, Type::Field, input.span())
                    }
                    (Some(t1), t2) => {
                        // Allow integer t2 magnitude (u8, u16, u32)
                        self.assert_magnitude_type(&t2, input.right.span());
                        self.assert_expected_type(expected, combined, t1, input.span())
                    }
                    (None, t2_type) => {
                        // Allow integer t2 magnitude (u8, u16, u32)
                        self.assert_magnitude_type(&t2_type, input.right.span());
                        t2
                    }
                }
            }
            BinaryOperation::Eq | BinaryOperation::Neq => {
                let t1 = self.visit_expression(&input.left, &None);
                let t2 = self.visit_expression(&input.right, &None);

                self.assert_eq_types(t1.as_ref().into(), t2.as_ref().into(), input.span());

                // Forces this to return a Boolean as the correct type output variation.
                t1.return_incorrect_type(&t2, &None).replace(Type::Boolean)
            }
            BinaryOperation::Lt | BinaryOperation::Gt | BinaryOperation::Lte | BinaryOperation::Gte => {
                // address, fields, int, scalar
                let t1 = self.visit_expression(&input.left, &None);
                let t1_type = t1.as_ref().into();
                self.assert_address_field_scalar_int_type(&t1_type, input.left.span());

                let t2 = self.visit_expression(&input.right, &None);
                let t2_type = t2.as_ref().into();
                self.assert_address_field_scalar_int_type(&t2_type, input.right.span());

                self.assert_eq_types(t1_type, t2_type, input.span());

                // Forces this to return a Boolean as the correct type output variation.
                t1.return_incorrect_type(&t2, &None).replace(Type::Boolean)
            }
            BinaryOperation::AddWrapped
            | BinaryOperation::SubWrapped
            | BinaryOperation::DivWrapped
            | BinaryOperation::MulWrapped => {
                // Assert equal integer types.
                self.assert_int_type(expected, input.span);
                let t1 = self.visit_expression(&input.left, expected);
                let t2 = self.visit_expression(&input.right, expected);

                t1.return_incorrect_type(&t2, expected)
            }
            BinaryOperation::Shl
            | BinaryOperation::ShlWrapped
            | BinaryOperation::Shr
            | BinaryOperation::ShrWrapped
            | BinaryOperation::PowWrapped => {
                // Assert left and expected are equal integer types.
                self.assert_int_type(expected, input.span);
                let t1 = self.visit_expression(&input.left, expected);

                // Assert right type is a magnitude (u8, u16, u32).
                let t2 = self.visit_expression(&input.right, &None);
                let t2_ty = t2.as_ref().into();
                self.assert_magnitude_type(&t2_ty, input.right.span());

                t1
            }
        }
    }

    fn visit_unary(&mut self, input: &'a UnaryExpression, expected: &Self::AdditionalInput) -> Self::Output {
        match input.op {
            UnaryOperation::Abs => {
                // Assert integer type only.
                self.assert_signed_int_type(expected, input.span());
                self.visit_expression(&input.receiver, expected)
            }
            UnaryOperation::AbsWrapped => {
                // Assert integer type only.
                self.assert_signed_int_type(expected, input.span());
                self.visit_expression(&input.receiver, expected)
            }
            UnaryOperation::Double => {
                // Assert field and group type only.
                self.assert_field_group_type(expected, input.span());
                self.visit_expression(&input.receiver, expected)
            }
            UnaryOperation::Inverse => {
                // Assert field type only.
                self.assert_expected_type(expected, TypeOutput::None, Type::Field, input.span());
                self.visit_expression(&input.receiver, expected)
            }
            UnaryOperation::Negate => {
                let prior_negate_state = self.negate;
                self.negate = !self.negate;

                let type_ = self.visit_expression(&input.receiver, expected);
                self.negate = prior_negate_state;
                match type_.as_ref().into() {
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
                        .emit_err(TypeCheckerError::type_is_not_negatable(t, input.receiver.span())),
                    _ => {}
                };
                type_
            }
            UnaryOperation::Not => {
                // Assert boolean, integer types only.
                self.assert_bool_int_type(expected, input.span());
                self.visit_expression(&input.receiver, expected)
            }
            UnaryOperation::Square => {
                // Assert field type only.
                self.assert_expected_type(expected, TypeOutput::None, Type::Field, input.span());
                self.visit_expression(&input.receiver, expected)
            }
            UnaryOperation::SquareRoot => {
                // Assert field and scalar types only.
                self.assert_field_scalar_type(expected, input.span());
                self.visit_expression(&input.receiver, expected)
            }
        }
    }

    fn visit_ternary(&mut self, input: &'a TernaryExpression, expected: &Self::AdditionalInput) -> Self::Output {
        self.visit_expression(&input.condition, &Some(Type::Boolean));

        let t1 = self.visit_expression(&input.if_true, expected);
        let t2 = self.visit_expression(&input.if_false, expected);

        t1.return_incorrect_type(&t2, &None)
    }

    fn visit_call(&mut self, input: &'a CallExpression, expected: &Self::AdditionalInput) -> Self::Output {
        match &*input.function {
            Expression::Identifier(ident) => {
                // The function symbol lookup is purposely done outside of the `if let` to avoid a RefCell lifetime bug in rust.
                // Don't move it into the `if let` or it will keep the `symbol_table` alive for the entire block and will be very memory inefficient!
                let f = self.symbol_table.borrow().lookup_fn(&ident.name).cloned();
                if let Some(func) = f {
                    let ret = if expected.is_some() {
                        self.assert_expected_option(func.type_, TypeOutput::None, expected, func.span)
                    } else {
                        // For now we assume that functions with no expected type return a mutable type.
                        // Can always be double checked during flattening.
                        TypeOutput::MutType(func.type_)
                    };

                    // Check number of function arguments.
                    if func.input.len() != input.arguments.len() {
                        self.handler.emit_err(TypeCheckerError::incorrect_num_args_to_call(
                            func.input.len(),
                            input.arguments.len(),
                            input.span(),
                        ));
                    }

                    // Check function argument types.
                    func.input
                        .iter()
                        .zip(input.arguments.iter())
                        .for_each(|(expected, argument)| {
                            self.visit_expression(argument, &Some(expected.get_variable().type_));
                        });

                    ret
                } else {
                    self.handler
                        .emit_err(TypeCheckerError::unknown_sym("function", &ident.name, ident.span()));
                    TypeOutput::None
                }
            }
            expr => self.visit_expression(expr, expected),
        }
    }

    fn visit_circuit_init(
        &mut self,
        input: &'a CircuitInitExpression,
        additional: &Self::AdditionalInput,
    ) -> Self::Output {
        let circ = self.symbol_table.borrow().lookup_circuit(&input.name.name).cloned();
        if let Some(circ) = circ {
            // Check circuit type name.
            self.assert_expected_circuit(circ.identifier, additional, input.name.span());

            // Check number of circuit members.
            if circ.members.len() != input.members.len() {
                self.handler.emit_err(TypeCheckerError::incorrect_num_circuit_members(
                    circ.members.len(),
                    input.members.len(),
                    input.span(),
                ));
            }

            // Set a dummy type for now.
            let mut output = TypeOutput::LitType(Type::Identifier(circ.identifier));
            let mut members = IndexMap::new();
            // Check circuit member types.
            for (name, expected) in circ.members.iter() {
                match expected {
                    CircuitMember::CircuitVariable(ident, type_) => {
                        // Lookup circuit variable name.
                        if let Some(actual) = input.members.get(name) {
                            let member_output = if let Some(expr) = &actual.expression {
                                self.visit_expression(expr, &Some(*type_))
                            } else if let Some(var) = self.symbol_table.borrow().lookup_variable(name) {
                                self.assert_expected_option(var.type_, var, &Some(*type_), input.span)
                            } else {
                                self.handler.emit_err(TypeCheckerError::unknown_sym(
                                    "variable",
                                    input.name,
                                    input.span(),
                                ));
                                return TypeOutput::None;
                            };

                            output = member_output.clone();
                            let member_value: Option<Value> = member_output.as_ref().into();
                            if let Some(member_value) = member_value {
                                members.insert(*name, member_value);
                            }
                        } else {
                            self.handler.emit_err(TypeCheckerError::unknown_sym(
                                "circuit member variable",
                                name,
                                ident.span(),
                            ));
                        };
                    }
                }
            }

            output.replace_value(Value::Circuit(circ.identifier, members))
        } else {
            self.handler.emit_err(TypeCheckerError::unknown_sym(
                "circuit",
                &input.name.name,
                input.name.span(),
            ));
            TypeOutput::None
        }
    }
}
