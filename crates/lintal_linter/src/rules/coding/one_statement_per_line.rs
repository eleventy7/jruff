//! OneStatementPerLine rule implementation.
//!
//! Checks that there is only one statement per line.
//!
//! Checkstyle equivalent: OneStatementPerLineCheck

use lintal_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use lintal_java_cst::CstNode;

use crate::{CheckContext, FromConfig, Properties, Rule};

/// Violation: multiple statements on same line.
#[derive(Debug, Clone)]
pub struct OneStatementPerLineViolation;

impl Violation for OneStatementPerLineViolation {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    fn message(&self) -> String {
        "Only one statement per line allowed.".to_string()
    }
}

/// Configuration for OneStatementPerLine rule.
#[derive(Debug, Clone)]
pub struct OneStatementPerLine {
    /// Whether try-with-resources resources count as statements.
    pub treat_try_resources_as_statement: bool,
}

impl Default for OneStatementPerLine {
    fn default() -> Self {
        Self {
            treat_try_resources_as_statement: false,
        }
    }
}

impl FromConfig for OneStatementPerLine {
    const MODULE_NAME: &'static str = "OneStatementPerLine";

    fn from_config(properties: &Properties) -> Self {
        let treat_try_resources_as_statement = properties
            .get("treatTryResourcesAsStatement")
            .map(|v| *v == "true")
            .unwrap_or(false);

        Self {
            treat_try_resources_as_statement,
        }
    }
}

impl Rule for OneStatementPerLine {
    fn name(&self) -> &'static str {
        "OneStatementPerLine"
    }

    fn check(&self, _ctx: &CheckContext, _node: &CstNode) -> Vec<Diagnostic> {
        // TODO: Implement
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintal_java_cst::TreeWalker;
    use lintal_java_parser::JavaParser;

    fn check_source(source: &str) -> Vec<Diagnostic> {
        let mut parser = JavaParser::new();
        let result = parser.parse(source).unwrap();
        let ctx = CheckContext::new(source);
        let rule = OneStatementPerLine::default();

        let mut diagnostics = vec![];
        for node in TreeWalker::new(result.tree.root_node(), source) {
            diagnostics.extend(rule.check(&ctx, &node));
        }
        diagnostics
    }

    #[test]
    fn test_two_statements_same_line() {
        let source = r#"
class Test {
    void method() {
        int a; int b;
    }
}
"#;
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1, "Expected 1 violation for two statements on same line");
    }

    #[test]
    fn test_single_statement_per_line_ok() {
        let source = r#"
class Test {
    void method() {
        int a;
        int b;
    }
}
"#;
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty(), "Single statements per line should not cause violations");
    }

    #[test]
    fn test_for_loop_header_ok() {
        let source = r#"
class Test {
    void method() {
        for (int i = 0; i < 10; i++) {}
    }
}
"#;
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty(), "For loop header should not cause violations");
    }
}
