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

use leo_ast::Program;
use leo_errors::{Result, Span};

use indexmap::IndexMap;

pub trait ImportResolver {
    fn resolve_package(&mut self, package_segments: &[&str], span: &Span) -> Result<Option<Program>>;
}

pub struct NullImportResolver;

impl ImportResolver for NullImportResolver {
    fn resolve_package(&mut self, _package_segments: &[&str], _span: &Span) -> Result<Option<Program>> {
        Ok(None)
    }
}

pub struct CoreImportResolver<'a, T: ImportResolver> {
    inner: &'a mut T,
}

impl<'a, T: ImportResolver> CoreImportResolver<'a, T> {
    pub fn new(inner: &'a mut T) -> Self {
        CoreImportResolver { inner }
    }
}

impl<'a, T: ImportResolver> ImportResolver for CoreImportResolver<'a, T> {
    fn resolve_package(&mut self, package_segments: &[&str], span: &Span) -> Result<Option<Program>> {
        if !package_segments.is_empty() && package_segments.get(0).unwrap() == &"core" {
            Ok(resolve_core_module(&*package_segments[1..].join("."))?)
        } else {
            self.inner.resolve_package(package_segments, span)
        }
    }
}

pub struct MockedImportResolver {
    pub packages: IndexMap<String, Program>,
}

impl ImportResolver for MockedImportResolver {
    fn resolve_package(&mut self, package_segments: &[&str], _span: &Span) -> Result<Option<Program>> {
        Ok(self.packages.get(&package_segments.join(".")).cloned())
    }
}

// TODO: Remove this.
pub fn load_ast(content: &str) -> Result<Program> {
    // Parses the Leo file and constructs a grammar ast.
    Ok(leo_parser::parse_ast("input.leo", content)?.into_repr())
}

// TODO: We should merge this with core
// TODO: Make asg deep copy so we can cache resolved core modules
// TODO: Figure out how to do headers without bogus returns
pub fn resolve_core_module(module: &str) -> Result<Option<Program>> {
    match module {
        "unstable.blake2s" => {
            let ast = load_ast(
                r#"
                circuit Blake2s {
                    function hash(seed: [u8; 32], message: [u8; 32]) -> [u8; 32] {
                        return [0; 32];
                    }
                }
                "#,
            )?;
            ast.set_core_mapping("blake2s");
            Ok(Some(ast))
        }
        _ => Ok(None),
    }
}
