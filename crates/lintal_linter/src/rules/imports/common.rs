//! Shared utilities for import rules.

use std::collections::HashSet;

use lintal_source_file::{LineIndex, SourceCode};
use lintal_text_size::{TextRange, TextSize};
use regex::Regex;
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
    if node.kind() == "import_declaration"
        && let Some(info) = parse_import_declaration(node, source, source_code)
    {
        imports.push(info);
    }

    // Only recurse into program-level nodes, not into class bodies
    if node.kind() == "program" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_declaration"
                && let Some(info) = parse_import_declaration(child, source, source_code)
            {
                imports.push(info);
            }
        }
    }
}

fn parse_import_declaration(
    node: Node,
    source: &str,
    source_code: &SourceCode,
) -> Option<ImportInfo> {
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

/// Collect all type identifiers used in the source code.
///
/// This traverses the AST and collects simple names of types that are referenced:
/// - Type identifiers in declarations, casts, generics
/// - Annotation names
/// - Static method call targets (for static imports)
pub fn collect_type_usages(root: Node, source: &str) -> HashSet<String> {
    let mut usages = HashSet::new();
    collect_usages_recursive(root, source, &mut usages);
    usages
}

/// Check if any ancestor of this node is a "defining" context where the identifier
/// is being declared rather than used.
fn has_defining_ancestor(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        match ancestor.kind() {
            // Import/package contexts - ALL identifiers here are not usages
            "import_declaration" | "package_declaration" => return true,

            // Type declarations - only the NAME is a definition
            "class_declaration"
            | "interface_declaration"
            | "enum_declaration"
            | "record_declaration"
            | "annotation_type_declaration" => {
                if let Some(name_node) = ancestor.child_by_field_name("name")
                    && is_ancestor_of(name_node, node)
                {
                    return true;
                }
            }

            // Method/constructor declarations - only the NAME is a definition
            "method_declaration" | "constructor_declaration" => {
                if let Some(name_node) = ancestor.child_by_field_name("name")
                    && is_ancestor_of(name_node, node)
                {
                    return true;
                }
            }

            // Variable/field/parameter declarations - the name being declared
            "variable_declarator" | "formal_parameter" | "catch_formal_parameter" => {
                if let Some(name_node) = ancestor.child_by_field_name("name")
                    && is_ancestor_of(name_node, node)
                {
                    return true;
                }
            }

            // Inferred lambda parameter (without type) - the parameter name is defined
            "inferred_parameters" => return true,

            // Enum constant - the name is being defined
            "enum_constant" => {
                if let Some(name_node) = ancestor.child_by_field_name("name")
                    && is_ancestor_of(name_node, node)
                {
                    return true;
                }
            }

            _ => {}
        }
        current = ancestor.parent();
    }
    false
}

/// Check if `ancestor` is an ancestor of `node` (or is `node` itself).
fn is_ancestor_of(ancestor: Node, node: Node) -> bool {
    if ancestor.id() == node.id() {
        return true;
    }
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.id() == ancestor.id() {
            return true;
        }
        current = parent.parent();
    }
    false
}

fn collect_usages_recursive(node: Node, source: &str, usages: &mut HashSet<String>) {
    match node.kind() {
        // Type identifier - used in declarations, generics, etc.
        "type_identifier" => {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                usages.insert(text.to_string());
            }
        }

        // Scoped type identifier - e.g., Map.Entry, use first part
        "scoped_type_identifier" => {
            // Get the first identifier (the imported type)
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        usages.insert(text.to_string());
                    }
                    break;
                }
            }
        }

        // Annotation - @Foo means Foo is used
        "marker_annotation" | "annotation" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        usages.insert(text.to_string());
                    }
                    break;
                }
                if child.kind() == "scoped_identifier" {
                    // @com.foo.Bar - get first identifier
                    if let Some(first) = child.child(0)
                        && let Ok(text) = first.utf8_text(source.as_bytes())
                    {
                        usages.insert(text.to_string());
                    }
                    break;
                }
            }
        }

        // Method invocation on a type - e.g., Arrays.sort()
        // Also captures direct static method calls - e.g., sort() from import static Arrays.sort
        "method_invocation" => {
            if let Some(object) = node.child_by_field_name("object") {
                // Has an object: e.g., Arrays.sort() - capture "Arrays"
                if object.kind() == "identifier"
                    && let Ok(text) = object.utf8_text(source.as_bytes())
                    && text.chars().next().is_some_and(|c| c.is_uppercase())
                {
                    usages.insert(text.to_string());
                }
            } else if let Some(name) = node.child_by_field_name("name")
                && let Ok(text) = name.utf8_text(source.as_bytes())
            {
                // No object: direct call like to() from import static Advice.to
                // Capture the method name for static import detection
                usages.insert(text.to_string());
            }
        }

        // Field access on a type - e.g., System.out
        "field_access" => {
            if let Some(object) = node.child_by_field_name("object")
                && object.kind() == "identifier"
                && let Ok(text) = object.utf8_text(source.as_bytes())
                && text.chars().next().is_some_and(|c| c.is_uppercase())
            {
                usages.insert(text.to_string());
            }
        }

        // Identifier in expression context - captures static field imports
        // e.g., SIZE_OF_INT from import static BitUtil.SIZE_OF_INT
        "identifier" => {
            // Skip if any ancestor is an import/package declaration or a defining context
            if !has_defining_ancestor(node)
                && let Some(parent) = node.parent()
            {
                // Also skip if this is the "name" field of a method_invocation with object
                // (those are handled separately above)
                let is_qualified_method_name = parent.kind() == "method_invocation"
                    && parent
                        .child_by_field_name("name")
                        .is_some_and(|n| n.id() == node.id())
                    && parent.child_by_field_name("object").is_some();

                if !is_qualified_method_name && let Ok(text) = node.utf8_text(source.as_bytes()) {
                    usages.insert(text.to_string());
                }
            }
        }

        _ => {}
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_usages_recursive(child, source, usages);
    }
}

/// Extract type references from Javadoc comments.
///
/// Parses:
/// - {@link Type}, {@link Type#method}, {@link Type#method(Param)}
/// - {@linkplain Type text}
/// - @see Type
/// - @throws Type, @exception Type
pub fn collect_javadoc_references(root: Node, source: &str) -> HashSet<String> {
    let mut references = HashSet::new();
    collect_javadoc_recursive(root, source, &mut references);
    references
}

fn collect_javadoc_recursive(node: Node, source: &str, references: &mut HashSet<String>) {
    if node.kind() == "block_comment"
        && let Ok(text) = node.utf8_text(source.as_bytes())
        && text.starts_with("/**")
    {
        parse_javadoc_types(text, references);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_javadoc_recursive(child, source, references);
    }
}

fn parse_javadoc_types(javadoc: &str, references: &mut HashSet<String>) {
    // Pattern for {@link Type}, {@link Type#method}, {@link Type#method(Param1, Param2)}
    // Also handles {@linkplain Type text} and {@link #method(Type)} (same-class method refs)
    lazy_static::lazy_static! {
        static ref LINK_RE: Regex = Regex::new(
            r"\{@(?:link|linkplain)\s+(?:([A-Z][A-Za-z0-9_]*))?(?:#[^}(]*(?:\(([^)]*)\))?)?[^}]*\}"
        ).unwrap();

        // @see Type, @see Type#method, @see fqn.Type#method(Param1, Param2)
        // Captures the reference which may include parentheses with spaces
        static ref SEE_RE: Regex = Regex::new(
            r"@see\s+([^\s(]+(?:\([^)]*\))?)"
        ).unwrap();

        static ref THROWS_RE: Regex = Regex::new(
            r"@(?:throws|exception)\s+([A-Z][A-Za-z0-9_.]*)"
        ).unwrap();

        static ref PARAM_TYPE_RE: Regex = Regex::new(
            r"([A-Z][A-Za-z0-9_]*)"
        ).unwrap();
    }

    // Extract from @link tags
    for cap in LINK_RE.captures_iter(javadoc) {
        if let Some(m) = cap.get(1) {
            references.insert(m.as_str().to_string());
        }
        // Also extract types from method parameters like Type#method(ParamType)
        if let Some(params) = cap.get(2) {
            for param_cap in PARAM_TYPE_RE.captures_iter(params.as_str()) {
                if let Some(m) = param_cap.get(1) {
                    references.insert(m.as_str().to_string());
                }
            }
        }
    }

    // Extract from @see tags - extract ALL capitalized identifiers
    // This handles: @see Type, @see fqn.Type#method, @see fqn.Type#method(Param)
    for cap in SEE_RE.captures_iter(javadoc) {
        if let Some(m) = cap.get(1) {
            // Extract all capitalized type names from the reference
            for type_cap in PARAM_TYPE_RE.captures_iter(m.as_str()) {
                if let Some(t) = type_cap.get(1) {
                    references.insert(t.as_str().to_string());
                }
            }
        }
    }

    // Extract from @throws/@exception tags
    for cap in THROWS_RE.captures_iter(javadoc) {
        if let Some(m) = cap.get(1) {
            let name = m.as_str().split('.').next().unwrap_or(m.as_str());
            references.insert(name.to_string());
        }
    }
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

    #[test]
    fn test_collect_type_usages_declaration() {
        let source = r#"
class Test {
    List<String> items;
}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let usages = collect_type_usages(result.tree.root_node(), source);

        assert!(usages.contains("List"));
        assert!(usages.contains("String"));
    }

    #[test]
    fn test_collect_type_usages_annotation() {
        let source = r#"
@Override
class Test {
    @Deprecated
    void method() {}
}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let usages = collect_type_usages(result.tree.root_node(), source);

        assert!(usages.contains("Override"));
        assert!(usages.contains("Deprecated"));
    }

    #[test]
    fn test_collect_type_usages_method_call() {
        let source = r#"
class Test {
    void method() {
        Arrays.sort(items);
    }
}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let usages = collect_type_usages(result.tree.root_node(), source);

        assert!(usages.contains("Arrays"));
    }

    #[test]
    fn test_collect_type_usages_inner_class() {
        let source = r#"
class Test {
    JToolBar.Separator sep;
}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let usages = collect_type_usages(result.tree.root_node(), source);

        assert!(
            usages.contains("JToolBar"),
            "Should capture outer class from inner class reference"
        );
    }

    #[test]
    fn test_javadoc_link() {
        let source = r#"
/**
 * See {@link List} for details.
 */
class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let refs = collect_javadoc_references(result.tree.root_node(), source);

        assert!(refs.contains("List"));
    }

    #[test]
    fn test_javadoc_link_with_method() {
        let source = r#"
/**
 * Uses {@link Arrays#sort(Object[])} internally.
 */
class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let refs = collect_javadoc_references(result.tree.root_node(), source);

        assert!(refs.contains("Arrays"));
        assert!(refs.contains("Object"));
    }

    #[test]
    fn test_javadoc_see() {
        let source = r#"
/**
 * @see Calendar
 */
class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let refs = collect_javadoc_references(result.tree.root_node(), source);

        assert!(refs.contains("Calendar"));
    }

    #[test]
    fn test_javadoc_throws() {
        let source = r#"
/**
 * @throws IOException if error
 * @exception RuntimeException if bad
 */
class Test {}
"#;
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();

        let refs = collect_javadoc_references(result.tree.root_node(), source);

        assert!(refs.contains("IOException"));
        assert!(refs.contains("RuntimeException"));
    }
}
