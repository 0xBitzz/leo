// Copyright (C) 2019-2021 Aleo Systems Inc.
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

use crate::errors::{AddressError, BooleanError, ConsoleError, ExpressionError, IntegerError, ValueError};
use leo_asg::Type;
use leo_ast::{FormattedError, LeoError, Span};

#[derive(Debug, Error)]
pub enum StatementError {
    #[error("{}", _0)]
    AddressError(#[from] AddressError),

    #[error("{}", _0)]
    BooleanError(#[from] BooleanError),

    #[error("{}", _0)]
    Error(#[from] FormattedError),

    #[error("{}", _0)]
    ExpressionError(#[from] ExpressionError),

    #[error("{}", _0)]
    IntegerError(#[from] IntegerError),

    #[error("{}", _0)]
    MacroError(#[from] ConsoleError),

    #[error("{}", _0)]
    ValueError(#[from] ValueError),
}

impl LeoError for StatementError {}

impl StatementError {
    fn new_from_span(message: String, span: &Span) -> Self {
        StatementError::Error(FormattedError::new_from_span(message, span))
    }

    pub fn array_assign_index(span: &Span) -> Self {
        let message = "Cannot assign single index to array of values".to_string();

        Self::new_from_span(message, span)
    }

    pub fn array_assign_index_const(span: &Span) -> Self {
        let message = "Cannot assign to non-const array index".to_string();

        Self::new_from_span(message, span)
    }

    pub fn array_assign_interior_index(span: &Span) -> Self {
        let message = "Cannot assign single index to interior of array of values".to_string();

        Self::new_from_span(message, span)
    }

    pub fn array_assign_range(span: &Span) -> Self {
        let message = "Cannot assign range of array values to single value".to_string();

        Self::new_from_span(message, span)
    }

    pub fn array_assign_index_bounds(index: usize, length: usize, span: &Span) -> Self {
        let message = format!(
            "Array assign index `{}` out of range for array of length `{}`",
            index, length
        );

        Self::new_from_span(message, span)
    }

    pub fn array_assign_range_order(start: usize, stop: usize, length: usize, span: &Span) -> Self {
        let message = format!(
            "Array assign range `{}`..`{}` out of range for array of length `{}`",
            start, stop, length
        );

        Self::new_from_span(message, span)
    }

    pub fn conditional_boolean(actual: String, span: &Span) -> Self {
        let message = format!("If, else conditional must resolve to a boolean, found `{}`", actual);

        Self::new_from_span(message, span)
    }

    pub fn invalid_number_of_definitions(expected: usize, actual: usize, span: &Span) -> Self {
        let message = format!(
            "Multiple definition statement expected {} return values, found {} values",
            expected, actual
        );

        Self::new_from_span(message, span)
    }

    pub fn multiple_definition(value: String, span: &Span) -> Self {
        let message = format!("cannot assign multiple variables to a single value: {}", value,);

        Self::new_from_span(message, span)
    }

    pub fn multiple_returns(span: &Span) -> Self {
        let message = "This function returns multiple times and produces unreachable circuits with undefined behavior."
            .to_string();

        Self::new_from_span(message, span)
    }

    pub fn no_returns(expected: &Type, span: &Span) -> Self {
        let message = format!(
            "function expected `{}` return type but no valid branches returned a result",
            expected
        );

        Self::new_from_span(message, span)
    }

    pub fn select_fail(first: String, second: String, span: &Span) -> Self {
        let message = format!(
            "Conditional select gadget failed to select between `{}` or `{}`",
            first, second
        );

        Self::new_from_span(message, span)
    }

    pub fn tuple_assign_index(span: &Span) -> Self {
        let message = "Cannot assign single index to tuple of values".to_string();

        Self::new_from_span(message, span)
    }

    pub fn tuple_assign_index_bounds(index: usize, length: usize, span: &Span) -> Self {
        let message = format!(
            "Tuple assign index `{}` out of range for tuple of length `{}`",
            index, length
        );

        Self::new_from_span(message, span)
    }

    pub fn unassigned(span: &Span) -> Self {
        let message = "Expected assignment of return values for expression".to_string();

        Self::new_from_span(message, span)
    }

    pub fn undefined_variable(name: String, span: &Span) -> Self {
        let message = format!("Attempted to assign to unknown variable `{}`", name);

        Self::new_from_span(message, span)
    }

    pub fn undefined_circuit(name: String, span: &Span) -> Self {
        let message = format!("Attempted to assign to unknown circuit `{}`", name);

        Self::new_from_span(message, span)
    }

    pub fn undefined_circuit_variable(name: String, span: &Span) -> Self {
        let message = format!("Attempted to assign to unknown circuit member variable `{}`", name);

        Self::new_from_span(message, span)
    }

    pub fn loop_index_const(span: &Span) -> Self {
        let message = "iteration range must be const".to_string();

        Self::new_from_span(message, span)
    }
}
