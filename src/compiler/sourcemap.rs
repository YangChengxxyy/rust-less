//! Source map generation support
//!
//! This module provides functionality to generate source maps for compiled CSS.

use crate::ast::Position;
use sourcemap::{RewriteOptions, SourceMap, SourceMapBuilder as SmBuilder};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Source map generator wrapper
pub struct SourceMapGenerator {
    builder: RefCell<SmBuilder>,
    file_ids: RefCell<HashMap<String, u32>>,
    output_file: RefCell<Option<String>>,
    source_root: RefCell<Option<String>>,
    source_base_path: RefCell<Option<PathBuf>>,
    lessjs_compat: RefCell<bool>,
    enabled: bool,
}

impl SourceMapGenerator {
    fn build_builder(output_file: Option<&str>, source_root: Option<&str>) -> SmBuilder {
        let mut builder = SmBuilder::new(output_file);
        if let Some(root) = source_root {
            builder.set_source_root(Some(root.to_string()));
        }
        builder
    }

    fn new_builder_from_config(&self) -> SmBuilder {
        let output_file = self.output_file.borrow().clone();
        let source_root = self.source_root.borrow().clone();
        Self::build_builder(output_file.as_deref(), source_root.as_deref())
    }

    /// Create a new source map generator
    pub fn new(enabled: bool) -> Self {
        Self {
            builder: RefCell::new(Self::build_builder(None, None)),
            file_ids: RefCell::new(HashMap::new()),
            output_file: RefCell::new(None),
            source_root: RefCell::new(None),
            source_base_path: RefCell::new(None),
            lessjs_compat: RefCell::new(false),
            enabled,
        }
    }

    /// Whether source map generation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set generated CSS file field in source map.
    pub fn set_file(&self, output_file: Option<&str>) {
        let output_file = output_file.map(|s| s.to_string());
        *self.output_file.borrow_mut() = output_file.clone();
        self.builder.borrow_mut().set_file(output_file);
    }

    /// Set sourceRoot field in source map.
    pub fn set_source_root(&self, source_root: Option<&str>) {
        let source_root = source_root.map(|s| s.to_string());
        *self.source_root.borrow_mut() = source_root.clone();
        self.builder.borrow_mut().set_source_root(source_root);
    }

    /// Set the base path used by less.js compatibility mode when rewriting source paths.
    pub fn set_source_base_path(&self, source_base_path: Option<PathBuf>) {
        *self.source_base_path.borrow_mut() = source_base_path;
    }

    /// Enable or disable less.js compatibility output mode.
    pub fn set_lessjs_compat_mode(&self, enabled: bool) {
        *self.lessjs_compat.borrow_mut() = enabled;
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
        let builder = self.builder.replace(self.new_builder_from_config());
        self.file_ids.borrow_mut().clear();
        Some(builder.into_sourcemap())
    }

    /// Reset internal state before a new compilation.
    pub fn reset(&self) {
        if !self.enabled {
            return;
        }
        self.builder.replace(self.new_builder_from_config());
        self.file_ids.borrow_mut().clear();
    }

    /// Generate the source map as a JSON string
    pub fn generate_json(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let mut sm = self.generate()?;

        if *self.lessjs_compat.borrow() {
            match sm.rewrite(&RewriteOptions {
                with_names: false,
                ..Default::default()
            }) {
                Ok(rewritten) => sm = rewritten,
                Err(err) => {
                    eprintln!("rust-less: source map rewrite failed: {}", err);
                    return None;
                }
            }
            self.rewrite_sources_for_lessjs_compat(&mut sm);
        }

        let mut out = Vec::new();
        if let Err(err) = sm.to_writer(&mut out) {
            eprintln!("rust-less: source map serialization failed: {}", err);
            return None;
        }
        match String::from_utf8(out) {
            Ok(json) => Some(json),
            Err(err) => {
                eprintln!("rust-less: source map JSON is not valid UTF-8: {}", err);
                None
            }
        }
    }

    fn rewrite_sources_for_lessjs_compat(&self, sm: &mut SourceMap) {
        // less.js writes root-prefixed entries directly into `sources` and does not emit `sourceRoot`.
        let source_root = self.source_root.borrow().clone();
        let source_base_path = self.source_base_path.borrow().clone();

        sm.set_source_root::<String>(None);

        let source_count = sm.get_source_count();
        for idx in 0..source_count {
            let Some(raw_source) = sm.get_source(idx) else {
                continue;
            };
            let normalized = normalize_source_for_lessjs(raw_source, source_base_path.as_deref());
            let with_root = apply_lessjs_source_root(source_root.as_deref(), &normalized);
            sm.set_source(idx, &with_root);
        }
    }
}

fn normalize_path_slashes(input: &str) -> String {
    input.replace('\\', "/")
}

fn normalize_source_for_lessjs(source: &str, source_base_path: Option<&Path>) -> String {
    let source_path = Path::new(source);
    if source_path.is_absolute() {
        if let Some(base_path) = source_base_path {
            if let Ok(relative_path) = source_path.strip_prefix(base_path) {
                let rel = normalize_path_slashes(&relative_path.to_string_lossy());
                if !rel.is_empty() {
                    return rel;
                }
            }
        }

        if let Some(file_name) = source_path.file_name() {
            return normalize_path_slashes(&file_name.to_string_lossy());
        }
    }

    normalize_path_slashes(source)
}

fn apply_lessjs_source_root(source_root: Option<&str>, source: &str) -> String {
    let Some(root) = source_root.map(str::trim).filter(|root| !root.is_empty()) else {
        return source.to_string();
    };

    if source.starts_with('/') || source.starts_with("http:") || source.starts_with("https:") {
        return source.to_string();
    }

    let root = root.trim_end_matches('/');
    if root.is_empty() {
        source.to_string()
    } else {
        format!("{}/{}", root, source.trim_start_matches('/'))
    }
}

impl Default for SourceMapGenerator {
    fn default() -> Self {
        Self::new(false)
    }
}
