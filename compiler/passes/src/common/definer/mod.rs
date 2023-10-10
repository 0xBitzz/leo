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

use leo_ast::{DeclarationType, DefinitionStatement, Expression, Identifier, NodeID, Statement, Type};
use leo_span::Symbol;

use std::{cell::RefCell, fmt::Display};

/// A struct used to create definition statements.
#[derive(Debug, Default, Clone)]
pub struct Definer {
    /// The inner counter.
    /// `RefCell` is used here to avoid `&mut` all over the compiler.
    inner: RefCell<DefinerInner>,
}

impl Definer {
    /// Return a new unique `Symbol` from a `&str`.
    pub fn unique_symbol(&self, arg: impl Display, separator: impl Display) -> Symbol {
        self.inner.borrow_mut().unique_symbol(arg, separator)
    }

    /// Constructs the definition statement `let place: type = expr;`.
    /// This function should be the only place where `DefinitionStatement`s are constructed.
    pub fn simple_definition_statement(
        &self,
        type_: Type,
        identifier: Identifier,
        value: Expression,
        id: NodeID,
    ) -> Statement {
        self.inner.borrow_mut().simple_definition_statement(type_, identifier, value, id)
    }
}

/// Contains the actual data for `Definer`.
/// Modeled this way to afford an API using interior mutability.
#[derive(Debug, Default, Clone)]
pub struct DefinerInner {
    /// A strictly increasing counter, used to ensure that new variable names are unique.
    pub(crate) counter: usize,
}

impl DefinerInner {
    /// Return a new unique `Symbol` from a `&str`.
    fn unique_symbol(&mut self, arg: impl Display, separator: impl Display) -> Symbol {
        self.counter += 1;
        Symbol::intern(&format!("{}{}{}", arg, separator, self.counter - 1))
    }

    /// Constructs the definition statement `place: type = expr;`.
    /// This function should be the only place where `DefinitionStatements`s are constructed.
    fn simple_definition_statement(
        &mut self,
        type_: Type,
        identifier: Identifier,
        value: Expression,
        id: NodeID,
    ) -> Statement {
        Statement::Definition(DefinitionStatement {
            declaration_type: DeclarationType::Let,
            type_,
            place: Expression::Identifier(identifier),
            value,
            span: Default::default(),
            id,
        })
    }
}
