//! MultipleVariableDeclarations rule implementation.
//!
//! Checks that each variable is declared in its own statement and on its own line.
//!
//! Checkstyle equivalent: MultipleVariableDeclarationsCheck

use lintal_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use lintal_java_cst::CstNode;

use crate::{CheckContext, FromConfig, Properties, Rule};

/// Violation: comma-separated variables in single declaration.
#[derive(Debug, Clone)]
pub struct MultipleInStatementViolation;

impl Violation for MultipleInStatementViolation {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    fn message(&self) -> String {
        "Each variable declaration must be in its own statement.".to_string()
    }
}

/// Violation: multiple declarations on same line.
#[derive(Debug, Clone)]
pub struct MultipleOnLineViolation;

impl Violation for MultipleOnLineViolation {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    fn message(&self) -> String {
        "Only one variable definition per line allowed.".to_string()
    }
}

/// Configuration for MultipleVariableDeclarations rule.
#[derive(Debug, Clone, Default)]
pub struct MultipleVariableDeclarations;

impl FromConfig for MultipleVariableDeclarations {
    const MODULE_NAME: &'static str = "MultipleVariableDeclarations";

    fn from_config(_properties: &Properties) -> Self {
        Self
    }
}

impl Rule for MultipleVariableDeclarations {
    fn name(&self) -> &'static str {
        "MultipleVariableDeclarations"
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
        let rule = MultipleVariableDeclarations;

        let mut diagnostics = vec![];
        for node in TreeWalker::new(result.tree.root_node(), source) {
            diagnostics.extend(rule.check(&ctx, &node));
        }
        diagnostics
    }

    #[test]
    fn test_comma_separated_violation() {
        let source = r#"
class Test {
    int i, j;
}
"#;
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1, "Expected 1 violation for comma-separated variables");
        assert!(diagnostics[0].kind.body.contains("own statement"));
    }

    #[test]
    fn test_same_line_violation() {
        let source = r#"
class Test {
    int i; int j;
}
"#;
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1, "Expected 1 violation for same-line declarations");
        assert!(diagnostics[0].kind.body.contains("per line"));
    }

    #[test]
    fn test_separate_lines_ok() {
        let source = r#"
class Test {
    int i;
    int j;
}
"#;
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty(), "Separate lines should not cause violations");
    }

    #[test]
    fn test_for_loop_ok() {
        let source = r#"
class Test {
    void method() {
        for (int i = 0, j = 0; i < 10; i++, j++) {}
    }
}
"#;
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty(), "For loop initializers should not cause violations");
    }
}
