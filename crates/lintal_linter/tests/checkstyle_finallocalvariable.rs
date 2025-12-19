//! Checkstyle compatibility tests for FinalLocalVariable rule.
//!
//! These tests verify that lintal produces the same violations as checkstyle
//! for the FinalLocalVariable check.

mod checkstyle_repo;

use lintal_java_cst::TreeWalker;
use lintal_java_parser::JavaParser;
use lintal_linter::rules::FinalLocalVariable;
use lintal_linter::{CheckContext, FromConfig, Rule};
use lintal_source_file::{LineIndex, SourceCode};
use std::collections::HashMap;

/// A violation at a specific location.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    line: usize,
    column: usize,
}

impl Violation {
    fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Run FinalLocalVariable rule on source and collect violations.
fn check_final_local_variable(source: &str, properties: HashMap<&str, &str>) -> Vec<Violation> {
    let mut parser = JavaParser::new();
    let Some(result) = parser.parse(source) else {
        panic!("Failed to parse source");
    };

    let rule = FinalLocalVariable::from_config(&properties);
    let ctx = CheckContext::new(source);
    let line_index = LineIndex::from_source_text(source);
    let source_code = SourceCode::new(source, &line_index);

    let mut violations = vec![];

    for node in TreeWalker::new(result.tree.root_node(), source) {
        let diagnostics = rule.check(&ctx, &node);
        for diagnostic in diagnostics {
            let loc = source_code.line_column(diagnostic.range.start());
            violations.push(Violation {
                line: loc.line.get(),
                column: loc.column.get(),
            });
        }
    }

    violations
}

/// Load a checkstyle test input file.
/// Returns None if the checkstyle repo is not available.
fn load_finallocalvariable_fixture(file_name: &str) -> Option<String> {
    let checkstyle_root = checkstyle_repo::checkstyle_repo()?;
    let path = checkstyle_root
        .join("src/test/resources/com/puppycrawl/tools/checkstyle/checks/coding/finallocalvariable")
        .join(file_name);
    std::fs::read_to_string(&path).ok()
}

/// Helper to verify violations match expected.
fn verify_violations(violations: &[Violation], expected: &[Violation]) {
    let mut missing = vec![];
    let mut unexpected = vec![];

    for exp in expected {
        let matched = violations
            .iter()
            .any(|v| v.line == exp.line && v.column == exp.column);

        if !matched {
            missing.push(exp.clone());
        }
    }

    for actual in violations {
        let matched = expected
            .iter()
            .any(|v| v.line == actual.line && v.column == actual.column);

        if !matched {
            unexpected.push(actual.clone());
        }
    }

    if !missing.is_empty() || !unexpected.is_empty() {
        println!("\n=== Violations Report ===");
        if !missing.is_empty() {
            println!("\nMissing violations:");
            for v in &missing {
                println!("  {}:{}", v.line, v.column);
            }
        }
        if !unexpected.is_empty() {
            println!("\nUnexpected violations:");
            for v in &unexpected {
                println!("  {}:{}", v.line, v.column);
            }
        }
        panic!("Violation mismatch detected");
    }
}

// =============================================================================
// Test: testInputFinalLocalVariableOne
// File: InputFinalLocalVariableOne.java
// Config: validateEnhancedForLoopVariable = (default)false
// Expected violations from checkstyle test:
//   17:13 - Variable 'i' should be declared final
//   17:16 - Variable 'j' should be declared final
//   19:18 - Variable 'runnable' should be declared final
//   29:13 - Variable 'i' should be declared final
//   33:13 - Variable 'z' should be declared final
//   35:16 - Variable 'obj' should be declared final
//   39:16 - Variable 'x' should be declared final
//   45:18 - Variable 'runnable' should be declared final
//   49:21 - Variable 'q' should be declared final
//   65:13 - Variable 'i' should be declared final
//   69:13 - Variable 'z' should be declared final
//   71:16 - Variable 'obj' should be declared final
//   75:16 - Variable 'x' should be declared final
//   83:21 - Variable 'w' should be declared final
//   85:26 - Variable 'runnable' should be declared final
// =============================================================================

#[test]
fn test_input_final_local_variable_one() {
    let Some(source) = load_finallocalvariable_fixture("InputFinalLocalVariableOne.java") else {
        eprintln!("Skipping test: checkstyle repo not available");
        return;
    };

    let properties = HashMap::new();
    let violations = check_final_local_variable(&source, properties);

    let expected = vec![
        Violation::new(17, 13),
        Violation::new(17, 16),
        Violation::new(19, 18),
        Violation::new(29, 13),
        Violation::new(33, 13),
        Violation::new(35, 16),
        Violation::new(39, 16),
        Violation::new(45, 18),
        Violation::new(49, 21),
        Violation::new(65, 13),
        Violation::new(69, 13),
        Violation::new(71, 16),
        Violation::new(75, 16),
        Violation::new(83, 21),
        Violation::new(85, 26),
    ];

    verify_violations(&violations, &expected);
}
