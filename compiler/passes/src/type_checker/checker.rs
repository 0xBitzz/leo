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

use crate::SymbolTable;

use leo_ast::{Identifier, IntegerType, Node, Type};
use leo_core::*;
use leo_errors::{emitter::Handler, TypeCheckerError};
use leo_span::{Span, Symbol};

use indexmap::IndexSet;
use std::cell::RefCell;

pub struct TypeChecker<'a> {
    pub(crate) symbol_table: RefCell<SymbolTable>,
    pub(crate) handler: &'a Handler,
    pub(crate) parent: Option<Symbol>,
    pub(crate) has_return: bool,
    pub(crate) negate: bool,
    pub(crate) account_types: IndexSet<Symbol>,
    pub(crate) algorithms_types: IndexSet<Symbol>,
}

const INT_TYPES: [Type; 10] = [
    Type::IntegerType(IntegerType::I8),
    Type::IntegerType(IntegerType::I16),
    Type::IntegerType(IntegerType::I32),
    Type::IntegerType(IntegerType::I64),
    Type::IntegerType(IntegerType::I128),
    Type::IntegerType(IntegerType::U8),
    Type::IntegerType(IntegerType::U16),
    Type::IntegerType(IntegerType::U32),
    Type::IntegerType(IntegerType::U64),
    Type::IntegerType(IntegerType::U128),
];

const SIGNED_INT_TYPES: [Type; 5] = [
    Type::IntegerType(IntegerType::I8),
    Type::IntegerType(IntegerType::I16),
    Type::IntegerType(IntegerType::I32),
    Type::IntegerType(IntegerType::I64),
    Type::IntegerType(IntegerType::I128),
];

const MAGNITUDE_TYPES: [Type; 3] = [
    Type::IntegerType(IntegerType::U8),
    Type::IntegerType(IntegerType::U16),
    Type::IntegerType(IntegerType::U32),
];

const fn create_type_superset<const S: usize, const A: usize, const O: usize>(
    subset: [Type; S],
    additional: [Type; A],
) -> [Type; O] {
    let mut superset: [Type; O] = [Type::IntegerType(IntegerType::U8); O];
    let mut i = 0;
    while i < S {
        superset[i] = subset[i];
        i += 1;
    }
    let mut j = 0;
    while j < A {
        superset[i + j] = additional[j];
        j += 1;
    }
    superset
}

const BOOL_INT_TYPES: [Type; 11] = create_type_superset(INT_TYPES, [Type::Boolean]);

const FIELD_INT_TYPES: [Type; 11] = create_type_superset(INT_TYPES, [Type::Field]);

const FIELD_GROUP_INT_TYPES: [Type; 12] = create_type_superset(FIELD_INT_TYPES, [Type::Group]);

const FIELD_GROUP_SCALAR_INT_TYPES: [Type; 13] = create_type_superset(FIELD_GROUP_INT_TYPES, [Type::Scalar]);

const FIELD_GROUP_TYPES: [Type; 2] = [Type::Field, Type::Group];

const FIELD_SCALAR_TYPES: [Type; 2] = [Type::Field, Type::Scalar];

impl<'a> TypeChecker<'a> {
    /// Returns a new type checker given a symbol table and error handler.
    pub fn new(symbol_table: SymbolTable, handler: &'a Handler) -> Self {
        Self {
            symbol_table: RefCell::new(symbol_table),
            handler,
            parent: None,
            has_return: false,
            negate: false,
            account_types: Account::types(),
            algorithms_types: Algorithms::types(),
        }
    }

    /// Emits a type checker error.
    pub(crate) fn emit_err(&self, err: TypeCheckerError) {
        self.handler.emit_err(err);
    }

    /// Emits an error if the given type conflicts with a core library type.
    pub(crate) fn check_core_type_conflict(&self, type_: &Option<Type>) {
        if let Some(Type::Identifier(ident)) = type_ {
            if self.account_types.contains(&ident.name) || self.algorithms_types.contains(&ident.name) {
                self.emit_err(TypeCheckerError::core_type_name_conflict(&ident.name, ident.span()));
            }
        }
    }

    /// Emits an error if the `circuit` is not a core library circuit.
    /// Emits an error if the `function` is not supported by the circuit.
    pub(crate) fn assert_core_circuit_call(&self, circuit: &Type, function: &Identifier) -> Option<CoreInstruction> {
        if let Type::Identifier(ident) = circuit {
            // Lookup core circuit
            match CoreInstruction::from_symbols(ident.name, function.name) {
                None => {
                    // Not a core library circuit.
                    self.emit_err(TypeCheckerError::invalid_core_instruction(
                        &ident.name,
                        function.name,
                        ident.span(),
                    ));
                }
                Some(core_circuit) => return Some(core_circuit),
            }
        }
        None
    }

    /// Emits an error if the two given types are not equal.
    pub(crate) fn assert_eq_types(&self, t1: Option<Type>, t2: Option<Type>, span: Span) {
        match (t1, t2) {
            (Some(t1), Some(t2)) if t1 != t2 => self.emit_err(TypeCheckerError::type_should_be(t1, t2, span)),
            (Some(type_), None) | (None, Some(type_)) => {
                self.emit_err(TypeCheckerError::type_should_be("no type", type_, span))
            }
            _ => {}
        }
    }

    /// Returns the `circuit` type and emits an error if the `expected` type does not match.
    pub(crate) fn assert_expected_circuit(&mut self, circuit: Identifier, expected: &Option<Type>, span: Span) -> Type {
        if let Some(Type::Identifier(expected)) = expected {
            if !circuit.matches(expected) {
                self.emit_err(TypeCheckerError::type_should_be(circuit.name, expected.name, span));
            }
        }

        Type::Identifier(circuit)
    }

    /// Returns the given `actual` type and emits an error if the `expected` type does not match.
    pub(crate) fn assert_expected_option(&self, actual: Type, expected: &Option<Type>, span: Span) -> Type {
        if let Some(expected) = expected {
            if !actual.eq_flat(expected) {
                self.emit_err(TypeCheckerError::type_should_be(actual, expected, span));
            }
        }

        actual
    }

    /// Returns the given `expected` type and emits an error if the `actual` type does not match.
    /// `span` should be the location of the expected type.
    pub(crate) fn assert_expected_type(&mut self, actual: &Option<Type>, expected: Type, span: Span) -> Type {
        if let Some(actual) = actual {
            if !actual.eq_flat(&expected) {
                self.emit_err(TypeCheckerError::type_should_be(actual, expected, span));
            }
        }

        expected
    }

    /// Emits an error to the error handler if the given type is not equal to any of the expected types.
    pub(crate) fn assert_one_of_types(&self, type_: &Option<Type>, expected: &[Type], span: Span) {
        if let Some(type_) = type_ {
            if !expected.iter().any(|t: &Type| t == type_) {
                self.emit_err(TypeCheckerError::expected_one_type_of(
                    expected.iter().map(|t| t.to_string() + ",").collect::<String>(),
                    type_,
                    span,
                ));
            }
        }
    }

    /// Emits an error to the handler if the given type is not a boolean or an integer.
    pub(crate) fn assert_bool_int_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &BOOL_INT_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a field or integer.
    pub(crate) fn assert_field_int_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &FIELD_INT_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a field or group.
    pub(crate) fn assert_field_group_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &FIELD_GROUP_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a field or scalar.
    pub(crate) fn assert_field_scalar_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &FIELD_SCALAR_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a field, group, or integer.
    pub(crate) fn assert_field_group_int_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &FIELD_GROUP_INT_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a field, group, scalar or integer.
    pub(crate) fn assert_field_group_scalar_int_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &FIELD_GROUP_SCALAR_INT_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not an integer.
    pub(crate) fn assert_int_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &INT_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a signed integer.
    pub(crate) fn assert_signed_int_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &SIGNED_INT_TYPES, span)
    }

    /// Emits an error to the handler if the given type is not a magnitude (u8, u16, u32).
    pub(crate) fn assert_magnitude_type(&self, type_: &Option<Type>, span: Span) {
        self.assert_one_of_types(type_, &MAGNITUDE_TYPES, span)
    }
}
