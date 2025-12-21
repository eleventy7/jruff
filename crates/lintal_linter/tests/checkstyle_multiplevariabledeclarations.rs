//! MultipleVariableDeclarations checkstyle compatibility tests.

mod checkstyle_repo;

use lintal_java_cst::TreeWalker;
use lintal_java_parser::JavaParser;
use lintal_linter::rules::MultipleVariableDeclarations;
use lintal_linter::{CheckContext, Rule};
use lintal_source_file::{LineIndex, SourceCode};

#[derive(Debug, Clone)]
struct Violation {
    line: usize,
    message_type: &'static str,
}

fn check_multiple_variable_declarations(source: &str) -> Vec<Violation> {
    let mut parser = JavaParser::new();
    let Some(result) = parser.parse(source) else {
        panic!("Failed to parse source");
    };

    let rule = MultipleVariableDeclarations;
    let ctx = CheckContext::new(source);
    let line_index = LineIndex::from_source_text(source);
    let source_code = SourceCode::new(source, &line_index);

    let mut violations = vec![];

    for node in TreeWalker::new(result.tree.root_node(), source) {
        let diagnostics = rule.check(&ctx, &node);
        for diagnostic in diagnostics {
            let loc = source_code.line_column(diagnostic.range.start());
            let msg_type = if diagnostic.kind.body.contains("own statement") {
                "comma"
            } else {
                "same_line"
            };
            violations.push(Violation {
                line: loc.line.get(),
                message_type: msg_type,
            });
        }
    }

    violations
}

fn load_fixture(file_name: &str) -> Option<String> {
    let path = checkstyle_repo::coding_test_input("multiplevariabledeclarations", file_name)?;
    std::fs::read_to_string(&path).ok()
}

#[test]
fn test_input_multiple_variable_declarations() {
    let Some(source) = load_fixture("InputMultipleVariableDeclarations.java") else {
        eprintln!("Skipping test: checkstyle repo not available");
        return;
    };

    let violations = check_multiple_variable_declarations(&source);

    // From checkstyle test file comments:
    // Line 11: int i, j; - comma violation
    // Line 12: int i1; int j1; - same line violation
    // Line 15: String str, str1; - comma violation
    // Line 16: Object obj; Object obj1; - same line violation
    // Line 20, 23: wrapped declarations - same line violations
    // Line 42: multiple on one line

    println!("Found {} violations:", violations.len());
    for v in &violations {
        println!("  Line {}: {}", v.line, v.message_type);
    }

    // Verify we detect comma-separated violations
    assert!(
        violations
            .iter()
            .any(|v| v.line == 11 && v.message_type == "comma"),
        "Should detect comma-separated on line 11"
    );

    // Verify we detect same-line violations
    assert!(
        violations
            .iter()
            .any(|v| v.line == 12 && v.message_type == "same_line"),
        "Should detect same-line on line 12"
    );
}

#[test]
fn test_for_loop_exception() {
    let source = r#"
class Test {
    void method() {
        for (int i = 0, j = 0; i < 10; i++, j++) {}
    }
}
"#;
    let violations = check_multiple_variable_declarations(source);
    assert!(
        violations.is_empty(),
        "For loop initializers should be exempt, got: {:?}",
        violations
    );
}
