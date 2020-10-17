// Copyright (C) 2019-2020 Aleo Systems Inc.
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
use crate::{Expression, ExpressionError, Frame};
use leo_static_check::Type;
use leo_typed::{Expression as UnresolvedExpression, Span};

impl Expression {
    /// Resolve a binary expression from left to right.
    /// If no expected type is given, then the expression resolves to the lhs type.
    pub(crate) fn binary(
        frame: &Frame,
        type_: &Type,
        lhs: UnresolvedExpression,
        rhs: UnresolvedExpression,
        _span: &Span,
    ) -> Result<(Self, Self), ExpressionError> {
        // Resolve lhs with expected type
        let lhs_resolved = Expression::new(frame, type_, lhs)?;

        // Resolve the rhs with expected type
        let rhs_resolved = Expression::new(frame, type_, rhs)?;

        Ok((lhs_resolved, rhs_resolved))
    }
}
