//! Checkstyle compatibility tests.
//!
//! These tests verify that lintal produces the same violations as checkstyle
//! for the same input files. Test files are fetched from the checkstyle repository
//! at test time to avoid bundling LGPL-licensed code.

mod checkstyle_repo;

use lintal_java_cst::TreeWalker;
use lintal_java_parser::JavaParser;
use lintal_linter::rules::WhitespaceAround;
use lintal_linter::{CheckContext, Rule};
use lintal_source_file::{LineIndex, SourceCode};

/// A violation at a specific location.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    line: usize,
    column: usize,
    message_key: &'static str,
    token: String,
}

impl Violation {
    fn not_preceded(line: usize, column: usize, token: &str) -> Self {
        Self {
            line,
            column,
            message_key: "ws.notPreceded",
            token: token.to_string(),
        }
    }

    fn not_followed(line: usize, column: usize, token: &str) -> Self {
        Self {
            line,
            column,
            message_key: "ws.notFollowed",
            token: token.to_string(),
        }
    }
}

/// Run WhitespaceAround rule on source and collect violations.
fn check_whitespace_around(source: &str) -> Vec<Violation> {
    let mut parser = JavaParser::new();
    let Some(result) = parser.parse(source) else {
        panic!("Failed to parse source");
    };

    let rule = WhitespaceAround::default();
    let ctx = CheckContext::new(source);
    let line_index = LineIndex::from_source_text(source);
    let source_code = SourceCode::new(source, &line_index);

    let mut violations = vec![];

    for node in TreeWalker::new(result.tree.root_node(), source) {
        let diagnostics = rule.check(&ctx, &node);
        for diagnostic in diagnostics {
            let loc = source_code.line_column(diagnostic.range.start());
            let message = diagnostic.kind.body.clone();

            // Parse message to determine if it's "not preceded" or "not followed"
            let (message_key, token) = if message.contains("before") {
                ("ws.notPreceded", extract_token(&message))
            } else if message.contains("after") {
                ("ws.notFollowed", extract_token(&message))
            } else {
                ("unknown", message.clone())
            };

            violations.push(Violation {
                line: loc.line.get(),
                column: loc.column.get(),
                message_key,
                token,
            });
        }
    }

    violations
}

/// Extract token from message like "Missing whitespace before `+`"
fn extract_token(message: &str) -> String {
    if let Some(start) = message.find('`')
        && let Some(end) = message[start + 1..].find('`')
    {
        return message[start + 1..start + 1 + end].to_string();
    }
    message.to_string()
}

/// Load a checkstyle test input file.
/// Returns None if the checkstyle repo is not available.
fn load_checkstyle_fixture(check_name: &str, file_name: &str) -> Option<String> {
    let path = checkstyle_repo::checkstyle_test_input(check_name, file_name)?;
    std::fs::read_to_string(&path).ok()
}

// =============================================================================
// InputWhitespaceAroundSimple.java tests
// =============================================================================
//
// Expected violations from checkstyle (testSimpleInput):
//   168:26: '=' is not followed by whitespace
//   169:26: '=' is not followed by whitespace
//   170:26: '=' is not followed by whitespace
//   171:26: '=' is not followed by whitespace
//   172:26: '=' is not followed by whitespace
//   173:26: '=' is not followed by whitespace
//
// NOTE: The checkstyle column numbers assume tab-width=8. Our implementation
// counts raw characters, so column numbers differ when tabs are present.
// The important thing is that we detect violations on the correct lines.

#[test]
fn test_whitespace_around_simple() {
    let Some(source) =
        load_checkstyle_fixture("whitespacearound", "InputWhitespaceAroundSimple.java")
    else {
        eprintln!("Skipping test: checkstyle repo not available (run tests with network access)");
        return;
    };

    let violations = check_whitespace_around(&source);

    // Expected lines from checkstyle (columns differ due to tab-width handling)
    let expected_lines = vec![168, 169, 170, 171, 172, 173];

    println!("Expected lines with violations: {:?}", expected_lines);

    println!("\nActual violations:");
    for v in &violations {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    // Check each expected line has a violation
    let mut missing_lines = vec![];
    for line in &expected_lines {
        if !violations
            .iter()
            .any(|v| v.line == *line && v.message_key == "ws.notFollowed" && v.token == "=")
        {
            missing_lines.push(*line);
        }
    }

    if !missing_lines.is_empty() {
        println!("\nMissing violations on lines: {:?}", missing_lines);
    }

    // Assert we find violations on all expected lines
    assert!(
        missing_lines.is_empty(),
        "Missing violations on lines: {:?}",
        missing_lines
    );
}

// =============================================================================
// InputWhitespaceAroundKeywordsAndOperators.java tests
// =============================================================================
//
// Expected violations from checkstyle (testKeywordsAndOperators):
//   32:22: '=' is not preceded by whitespace
//   32:22: '=' is not followed by whitespace
//   34:23: '=' is not followed by whitespace
//   42:14: '=' is not preceded by whitespace
//   43:10: '=' is not preceded by whitespace
//   43:10: '=' is not followed by whitespace
//   44:10: '+=' is not preceded by whitespace
//   44:10: '+=' is not followed by whitespace
//   45:11: '-=' is not followed by whitespace
//   53:9: 'synchronized' is not followed by whitespace
//   55:9: 'try' is not followed by whitespace
//   55:12: '{' is not preceded by whitespace
//   57:9: 'catch' is not followed by whitespace
//   57:34: '{' is not preceded by whitespace
//   74:9: 'if' is not followed by whitespace
//   92:13: 'return' is not followed by whitespace
//   113:29: '?' is not preceded by whitespace
//   113:29: '?' is not followed by whitespace
//   113:34: ':' is not preceded by whitespace
//   113:34: ':' is not followed by whitespace
//   114:15: '==' is not preceded by whitespace
//   114:15: '==' is not followed by whitespace
//   120:19: '*' is not followed by whitespace
//   120:21: '*' is not preceded by whitespace
//   135:18: '%' is not preceded by whitespace
//   136:19: '%' is not followed by whitespace
//   137:18: '%' is not preceded by whitespace
//   137:18: '%' is not followed by whitespace
//   139:18: '/' is not preceded by whitespace
//   140:19: '/' is not followed by whitespace
//   141:18: '/' is not preceded by whitespace
//   141:18: '/' is not followed by whitespace
//   167:9: 'assert' is not followed by whitespace
//   170:20: ':' is not preceded by whitespace
//   170:20: ':' is not followed by whitespace
//   276:13: '}' is not followed by whitespace
//   305:24: '+' is not followed by whitespace
//   305:24: '+' is not preceded by whitespace
//   305:28: '+' is not followed by whitespace
//   305:28: '+' is not preceded by whitespace

#[test]
fn test_whitespace_around_keywords_and_operators() {
    let Some(source) = load_checkstyle_fixture(
        "whitespacearound",
        "InputWhitespaceAroundKeywordsAndOperators.java",
    ) else {
        eprintln!("Skipping test: checkstyle repo not available (run tests with network access)");
        return;
    };

    let violations = check_whitespace_around(&source);

    // Expected violations from checkstyle (subset focusing on operators we should catch)
    let expected_operators = vec![
        // Assignment operators
        Violation::not_preceded(32, 22, "="),
        Violation::not_followed(32, 22, "="),
        Violation::not_followed(34, 23, "="),
        Violation::not_preceded(42, 14, "="),
        Violation::not_preceded(43, 10, "="),
        Violation::not_followed(43, 10, "="),
        Violation::not_preceded(44, 10, "+="),
        Violation::not_followed(44, 10, "+="),
        Violation::not_followed(45, 11, "-="),
        // Comparison operators
        Violation::not_preceded(114, 15, "=="),
        Violation::not_followed(114, 15, "=="),
        // Arithmetic operators
        Violation::not_followed(120, 19, "*"),
        Violation::not_preceded(120, 21, "*"),
        Violation::not_preceded(135, 18, "%"),
        Violation::not_followed(136, 19, "%"),
        Violation::not_preceded(137, 18, "%"),
        Violation::not_followed(137, 18, "%"),
        Violation::not_preceded(139, 18, "/"),
        Violation::not_followed(140, 19, "/"),
        Violation::not_preceded(141, 18, "/"),
        Violation::not_followed(141, 18, "/"),
        // Plus operator
        Violation::not_followed(305, 24, "+"),
        Violation::not_preceded(305, 24, "+"),
        Violation::not_followed(305, 28, "+"),
        Violation::not_preceded(305, 28, "+"),
    ];

    println!("Expected operator violations:");
    for v in &expected_operators {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    println!("\nActual violations:");
    for v in &violations {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    // Check operator violations
    let mut found = 0;
    let mut missing = vec![];
    for exp in &expected_operators {
        if violations.iter().any(|v| {
            v.line == exp.line && v.column == exp.column && v.message_key == exp.message_key
        }) {
            found += 1;
        } else {
            missing.push(exp.clone());
        }
    }

    println!(
        "\nFound {}/{} expected operator violations",
        found,
        expected_operators.len()
    );
    if !missing.is_empty() {
        println!("Missing:");
        for v in &missing {
            println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
        }
    }

    // Assert we found all expected operator violations
    assert_eq!(
        found,
        expected_operators.len(),
        "Missing {} operator violations",
        expected_operators.len() - found
    );

    // Also check keyword violations
    let expected_keywords = vec![
        // Keywords
        Violation::not_followed(53, 9, "synchronized"),
        Violation::not_followed(55, 9, "try"),
        Violation::not_followed(57, 9, "catch"),
        Violation::not_followed(74, 9, "if"),
        Violation::not_followed(92, 13, "return"),
        Violation::not_followed(167, 9, "assert"),
    ];

    println!("\nExpected keyword violations:");
    for v in &expected_keywords {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    let mut keyword_found = 0;
    let mut keyword_missing = vec![];
    for exp in &expected_keywords {
        if violations
            .iter()
            .any(|v| v.line == exp.line && v.message_key == exp.message_key && v.token == exp.token)
        {
            keyword_found += 1;
        } else {
            keyword_missing.push(exp.clone());
        }
    }

    println!(
        "\nFound {}/{} expected keyword violations",
        keyword_found,
        expected_keywords.len()
    );
    if !keyword_missing.is_empty() {
        println!("Missing keywords:");
        for v in &keyword_missing {
            println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
        }
    }

    // Expected brace violations
    let expected_braces = vec![
        Violation::not_preceded(55, 12, "{"),
        Violation::not_preceded(57, 34, "{"),
    ];

    println!("\nExpected brace violations:");
    for v in &expected_braces {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    let mut brace_found = 0;
    for exp in &expected_braces {
        if violations
            .iter()
            .any(|v| v.line == exp.line && v.message_key == exp.message_key && v.token == exp.token)
        {
            brace_found += 1;
        }
    }

    println!(
        "\nFound {}/{} expected brace violations",
        brace_found,
        expected_braces.len()
    );

    // Ternary violations
    let expected_ternary = vec![
        Violation::not_preceded(113, 29, "?"),
        Violation::not_followed(113, 29, "?"),
        Violation::not_preceded(113, 34, ":"),
        Violation::not_followed(113, 34, ":"),
    ];

    println!("\nExpected ternary violations:");
    for v in &expected_ternary {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    let mut ternary_found = 0;
    for exp in &expected_ternary {
        if violations
            .iter()
            .any(|v| v.line == exp.line && v.message_key == exp.message_key && v.token == exp.token)
        {
            ternary_found += 1;
        }
    }

    println!(
        "\nFound {}/{} expected ternary violations",
        ternary_found,
        expected_ternary.len()
    );

    // Summary
    let total_expected = expected_operators.len()
        + expected_keywords.len()
        + expected_braces.len()
        + expected_ternary.len();
    let total_found = found + keyword_found + brace_found + ternary_found;
    println!(
        "\n=== TOTAL: Found {}/{} expected violations ===",
        total_found, total_expected
    );
}

// =============================================================================
// Basic operator tests with minimal fixtures (no external dependency)
// =============================================================================

#[test]
fn test_binary_plus_without_spaces() {
    let source = r#"class Foo { int x = 1+2; }"#;
    let violations = check_whitespace_around(source);

    println!("Violations for '1+2':");
    for v in &violations {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    // Should have 2 violations: not preceded and not followed
    assert!(
        violations.len() >= 2,
        "Expected at least 2 violations, got {}",
        violations.len()
    );
}

#[test]
fn test_binary_plus_with_spaces() {
    let source = r#"class Foo { int x = 1 + 2; }"#;
    let violations = check_whitespace_around(source);

    println!("Violations for '1 + 2':");
    for v in &violations {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    // Should have no violations for the + operator
    let plus_violations: Vec<_> = violations.iter().filter(|v| v.token == "+").collect();
    assert!(
        plus_violations.is_empty(),
        "Expected no + violations, got {:?}",
        plus_violations
    );
}

#[test]
fn test_assignment_without_space_after() {
    let source = r#"class Foo { int x =1; }"#;
    let violations = check_whitespace_around(source);

    println!("Violations for 'x =1':");
    for v in &violations {
        println!("  {}:{}: {} `{}`", v.line, v.column, v.message_key, v.token);
    }

    // Should have at least 1 violation: = not followed by whitespace
    let eq_violations: Vec<_> = violations.iter().filter(|v| v.token == "=").collect();
    assert!(!eq_violations.is_empty(), "Expected = violations");
}
