//! Source map generation support
//!
//! This module provides functionality to generate source maps for compiled CSS.

use crate::ast::Position;
use sourcemap::{SourceMap, SourceMapBuilder as SmBuilder};
use std::cell::RefCell;
use std::collections::HashMap;

/// Source map generator wrapper
pub struct SourceMapGenerator {
    builder: RefCell<SmBuilder>,
    file_ids: RefCell<HashMap<String, u32>>,
    enabled: bool,
}

impl SourceMapGenerator {
    /// Create a new source map generator
    pub fn new(enabled: bool) -> Self {
        Self {
            builder: RefCell::new(SmBuilder::new(None)),
            file_ids: RefCell::new(HashMap::new()),
            enabled,
        }
    }

    /// Add a mapping between source position and generated position
    ///
    /// * `source_file` - Path to the source file
    /// * `source_pos` - Position in the source file
    /// * `gen_line` - Line number in generated CSS (0-based)
    /// * `gen_col` - Column number in generated CSS (0-based)
    /// * `name` - Optional name associated with this mapping
    pub fn add_mapping(
        &self,
        source_file: &str,
        source_pos: &Position,
        gen_line: u32,
        gen_col: u32,
        name: Option<&str>,
    ) {
        if !self.enabled {
            return;
        }

        let mut builder = self.builder.borrow_mut();
        let mut file_ids = self.file_ids.borrow_mut();

        let file_id = if let Some(id) = file_ids.get(source_file) {
            *id
        } else {
            let id = builder.add_source(source_file);
            file_ids.insert(source_file.to_string(), id);
            id
        };

        // AST positions are 1-based, sourcemap expects 0-based
        let src_line = source_pos.line.saturating_sub(1) as u32;
        let src_col = source_pos.column.saturating_sub(1) as u32;

        let name_id = name.map(|n| builder.add_name(n));

        builder.add_raw(
            gen_line,
            gen_col,
            src_line,
            src_col,
            Some(file_id),
            name_id,
            false,
        );
    }

    /// Generate the source map
    pub fn generate(&self) -> Option<SourceMap> {
        if !self.enabled {
            return None;
        }
        // SourceMapBuilder doesn't support Clone, and into_sourcemap consumes it.
        // We replace it with a new builder.
        let builder = self.builder.replace(SmBuilder::new(None));
        self.file_ids.borrow_mut().clear();
        Some(builder.into_sourcemap())
    }

    /// Reset internal state before a new compilation.
    pub fn reset(&self) {
        if !self.enabled {
            return;
        }
        self.builder.replace(SmBuilder::new(None));
        self.file_ids.borrow_mut().clear();
    }

    /// Generate the source map as a JSON string
    pub fn generate_json(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let sm = self.generate()?;
        let mut out = Vec::new();
        sm.to_writer(&mut out).ok()?;
        String::from_utf8(out).ok()
    }
}

impl Default for SourceMapGenerator {
    fn default() -> Self {
        Self::new(false)
    }
}
