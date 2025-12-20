//! Shared utilities for import rules.

use lintal_source_file::{LineIndex, SourceCode};
use lintal_text_size::{TextRange, TextSize};
use tree_sitter::Node;

/// Represents a parsed import statement.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Full import path, e.g., "java.util.List" or "java.util.*"
    pub path: String,
    /// Simple name for non-wildcard imports, e.g., "List"
    pub simple_name: Option<String>,
    /// Whether this is a static import
    pub is_static: bool,
    /// Whether this ends with .*
    pub is_wildcard: bool,
    /// Source range for the import declaration
    pub range: TextRange,
    /// Line number (1-indexed) for duplicate detection
    pub line: usize,
}

impl ImportInfo {
    /// Get the package part of the import path (everything before the last dot).
    pub fn package(&self) -> Option<&str> {
        if self.is_wildcard {
            // For "java.util.*", package is "java.util"
            Some(&self.path[..self.path.len() - 2])
        } else {
            // For "java.util.List", package is "java.util"
            self.path.rfind('.').map(|i| &self.path[..i])
        }
    }
}

/// Collect all import declarations from the source.
pub fn collect_imports(root: Node, source: &str, line_index: &LineIndex) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    let source_code = SourceCode::new(source, line_index);

    collect_imports_recursive(root, source, &source_code, &mut imports);
    imports
}

fn collect_imports_recursive(
    node: Node,
    source: &str,
    source_code: &SourceCode,
    imports: &mut Vec<ImportInfo>,
) {
    if node.kind() == "import_declaration" {
        if let Some(info) = parse_import_declaration(node, source, source_code) {
            imports.push(info);
        }
    }

    // Only recurse into program-level nodes, not into class bodies
    if node.kind() == "program" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_declaration" {
                if let Some(info) = parse_import_declaration(child, source, source_code) {
                    imports.push(info);
                }
            }
        }
    }
}

fn parse_import_declaration(node: Node, source: &str, source_code: &SourceCode) -> Option<ImportInfo> {
    let start = TextSize::from(node.start_byte() as u32);
    let end = TextSize::from(node.end_byte() as u32);
    let range = TextRange::new(start, end);
    let line = source_code.line_column(start).line.get();

    let mut is_static = false;
    let mut path_parts = Vec::new();
    let mut is_wildcard = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "static" => is_static = true,
            "asterisk" => is_wildcard = true,
            "identifier" | "scoped_identifier" => {
                path_parts.push(child.utf8_text(source.as_bytes()).ok()?);
            }
            _ => {}
        }
    }

    if path_parts.is_empty() {
        return None;
    }

    let mut path = path_parts.join(".");
    if is_wildcard {
        path.push_str(".*");
    }

    let simple_name = if is_wildcard {
        None
    } else {
        path.rsplit('.').next().map(String::from)
    };

    Some(ImportInfo {
        path,
        simple_name,
        is_static,
        is_wildcard,
        range,
        line,
    })
}

/// Extract the package name from the source file.
pub fn get_package_name(root: Node, source: &str) -> Option<String> {
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "package_declaration" {
            return extract_package_path(child, source);
        }
    }
    None
}

fn extract_package_path(node: Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
            return child.utf8_text(source.as_bytes()).ok().map(String::from);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintal_java_parser::JavaParser;
    use lintal_source_file::LineIndex;

    #[test]
    fn test_collect_simple_import() {
        let source = r#"
import java.util.List;

class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();
        let line_index = LineIndex::from_source_text(source);

        let imports = collect_imports(result.tree.root_node(), source, &line_index);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "java.util.List");
        assert_eq!(imports[0].simple_name, Some("List".to_string()));
        assert!(!imports[0].is_static);
        assert!(!imports[0].is_wildcard);
    }

    #[test]
    fn test_collect_wildcard_import() {
        let source = r#"
import java.util.*;

class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();
        let line_index = LineIndex::from_source_text(source);

        let imports = collect_imports(result.tree.root_node(), source, &line_index);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "java.util.*");
        assert!(imports[0].simple_name.is_none());
        assert!(imports[0].is_wildcard);
    }

    #[test]
    fn test_collect_static_import() {
        let source = r#"
import static java.lang.Math.PI;

class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();
        let line_index = LineIndex::from_source_text(source);

        let imports = collect_imports(result.tree.root_node(), source, &line_index);

        assert_eq!(imports.len(), 1);
        assert!(imports[0].is_static);
        assert_eq!(imports[0].simple_name, Some("PI".to_string()));
    }

    #[test]
    fn test_get_package_name() {
        let source = r#"
package com.example.myapp;

import java.util.List;

class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let package = get_package_name(result.tree.root_node(), source);

        assert_eq!(package, Some("com.example.myapp".to_string()));
    }

    #[test]
    fn test_no_package() {
        let source = r#"
import java.util.List;

class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let package = get_package_name(result.tree.root_node(), source);

        assert!(package.is_none());
    }
}
