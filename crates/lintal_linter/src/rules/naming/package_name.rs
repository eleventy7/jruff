//! PackageName rule implementation.
//!
//! Checks that package names conform to a specified pattern.

use lintal_diagnostics::{Diagnostic, FixAvailability, Violation};
use lintal_java_cst::CstNode;
use lintal_text_size::TextRange;
use regex::Regex;

use crate::{CheckContext, FromConfig, Properties, Rule};

/// Default pattern for package names: starts with lowercase, followed by dot-separated segments
const DEFAULT_FORMAT: &str = r"^[a-z]+(\.[a-zA-Z_]\w*)*$";

/// Node kinds that represent package declarations
const RELEVANT_KINDS: &[&str] = &["package_declaration"];

/// Configuration for PackageName rule.
#[derive(Debug, Clone)]
pub struct PackageName {
    /// Regex pattern for valid package names
    format: Regex,
    /// Format string for error messages
    format_str: String,
}

impl Default for PackageName {
    fn default() -> Self {
        Self {
            format: Regex::new(DEFAULT_FORMAT).unwrap(),
            format_str: DEFAULT_FORMAT.to_string(),
        }
    }
}

impl FromConfig for PackageName {
    const MODULE_NAME: &'static str = "PackageName";

    fn from_config(properties: &Properties) -> Self {
        let format_str = properties
            .get("format")
            .copied()
            .unwrap_or(DEFAULT_FORMAT)
            .to_string();

        let format =
            Regex::new(&format_str).unwrap_or_else(|_| Regex::new(DEFAULT_FORMAT).unwrap());

        Self { format, format_str }
    }
}

/// Violation for package name not matching pattern.
#[derive(Debug, Clone)]
pub struct PackageNameInvalid {
    pub name: String,
    pub pattern: String,
}

impl Violation for PackageNameInvalid {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::None;

    fn message(&self) -> String {
        format!(
            "Name '{}' must match pattern '{}'.",
            self.name, self.pattern
        )
    }
}

impl Rule for PackageName {
    fn name(&self) -> &'static str {
        "PackageName"
    }

    fn relevant_kinds(&self) -> &'static [&'static str] {
        RELEVANT_KINDS
    }

    fn check(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        // Only check package_declaration nodes
        if node.kind() != "package_declaration" {
            return vec![];
        }

        // Get the package name from the scoped_identifier or identifier child
        let package_name = self.extract_package_name(ctx, node);

        if let Some((name, range)) = package_name {
            // Check against pattern
            if !self.format.is_match(&name) {
                return vec![Diagnostic::new(
                    PackageNameInvalid {
                        name,
                        pattern: self.format_str.clone(),
                    },
                    range,
                )];
            }
        }

        vec![]
    }
}

impl PackageName {
    /// Extract the package name and its range from a package declaration.
    fn extract_package_name(
        &self,
        ctx: &CheckContext,
        node: &CstNode,
    ) -> Option<(String, TextRange)> {
        // The package declaration contains either a scoped_identifier or an identifier
        for child in node.children() {
            match child.kind() {
                "scoped_identifier" | "identifier" => {
                    let name = ctx.source()[child.range()].to_string();
                    return Some((name, child.range()));
                }
                _ => continue,
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintal_java_cst::TreeWalker;
    use lintal_java_parser::JavaParser;

    fn check_source(source: &str, properties: Properties) -> Vec<Diagnostic> {
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();
        let ctx = CheckContext::new(source);
        let rule = PackageName::from_config(&properties);

        let mut diagnostics = vec![];
        for node in TreeWalker::new(result.tree.root_node(), source) {
            diagnostics.extend(rule.check(&ctx, &node));
        }
        diagnostics
    }

    #[test]
    fn test_valid_package_name() {
        let source = "package com.example.mypackage;";
        let diagnostics = check_source(source, Properties::new());
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_valid_simple_package() {
        let source = "package example;";
        let diagnostics = check_source(source, Properties::new());
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_invalid_uppercase_start() {
        let source = "package Com.example;";
        let diagnostics = check_source(source, Properties::new());
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_custom_format() {
        let source = "package com.example.test;";
        let mut properties = Properties::new();
        properties.insert("format", "[A-Z]+");
        let diagnostics = check_source(source, properties);
        assert_eq!(diagnostics.len(), 1); // Doesn't match uppercase pattern
    }

    #[test]
    fn test_custom_format_matches() {
        let source = "package EXAMPLE;";
        let mut properties = Properties::new();
        properties.insert("format", "^[A-Z]+$");
        let diagnostics = check_source(source, properties);
        assert_eq!(diagnostics.len(), 0); // Matches uppercase pattern
    }
}
