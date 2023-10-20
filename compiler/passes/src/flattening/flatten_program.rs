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

use leo_ast::{Finalize, Function, ProgramReconstructor, StatementReconstructor, Type};

impl ProgramReconstructor for Flattener<'_> {
    /// Flattens a function's body and finalize block, if it exists.
    fn reconstruct_function(&mut self, function: Function) -> Function {
        // First, flatten the finalize block. This allows us to initialize self.finalizes correctly.
        // Note that this is safe since the finalize block is independent of the function body.
        let finalize = function.finalize.map(|finalize| {
            // Initialize `self.structs` with the finalize's input as necessary.
            self.structs = Default::default();
            for input in &finalize.input {
                if let Type::Identifier(struct_name) = input.type_() {
                    // Note that this unwrap is safe since type checking guarantees that the struct exists.
                    let struct_ = self.symbol_table.lookup_struct(struct_name.name).unwrap().clone();
                    self.structs.insert(input.identifier().name, struct_);
                }
            }
            // Flatten the finalize block.
            let mut block = self.reconstruct_block(finalize.block).0;

            // Get all of the guards and return expression.
            let returns = self.clear_early_returns();

            // Fold the return statements into the block.
            self.fold_returns(&mut block, returns);

            Finalize {
                identifier: finalize.identifier,
                input: finalize.input,
                output: finalize.output,
                output_type: finalize.output_type,
                block,
                span: finalize.span,
                id: finalize.id,
            }
        });

        // Initialize `self.structs` with the function's input as necessary.
        self.structs = Default::default();
        for input in &function.input {
            if let Type::Identifier(struct_name) = input.type_() {
                // Note that this unwrap is safe since type checking guarantees that the struct exists.
                let struct_ = self.symbol_table.lookup_struct(struct_name.name).unwrap().clone();
                self.structs.insert(input.identifier().name, struct_);
            }
        }

        // Flatten the function body.
        let mut block = self.reconstruct_block(function.block).0;

        // Get all of the guards and return expression.
        let returns = self.clear_early_returns();

        // Fold the return statements into the block.
        self.fold_returns(&mut block, returns);

        Function {
            annotations: function.annotations,
            variant: function.variant,
            identifier: function.identifier,
            input: function.input,
            output: function.output,
            output_type: function.output_type,
            block,
            finalize,
            span: function.span,
            id: function.id,
        }
    }
}
