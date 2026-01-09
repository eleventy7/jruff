//! Test utilities for naming convention rules.
//!
//! This module provides a metrics-based test framework for validating
//! naming convention rules against checkstyle's test suite.
//!
//! ## Metrics Tracked
//! - **Detected**: Violations found by lintal that match checkstyle expectations (true positives)
//! - **Missed**: Violations expected by checkstyle but not found by lintal (false negatives)
//! - **Extra**: Violations found by lintal but not expected by checkstyle (false positives)

#![allow(clippy::collapsible_if)]

use lintal_java_cst::TreeWalker;
use lintal_java_parser::JavaParser;
use lintal_linter::{CheckContext, FromConfig, Properties, Rule};
use lintal_source_file::{LineIndex, SourceCode};
use regex::Regex;
use std::path::PathBuf;

/// A violation at a specific location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Violation {
    pub line: usize,
    pub column: usize,
    pub name: Option<String>,
}

impl Violation {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            name: None,
        }
    }

    pub fn with_name(line: usize, column: usize, name: &str) -> Self {
        Self {
            line,
            column,
            name: Some(name.to_string()),
        }
    }
}

/// Result of comparing lintal output against checkstyle expectations.
#[derive(Debug, Clone, Default)]
pub struct TestMetrics {
    /// Test case name
    pub test_name: String,
    /// Violations correctly detected (true positives)
    pub detected: Vec<Violation>,
    /// Violations missed by lintal (false negatives)
    pub missed: Vec<Violation>,
    /// Extra violations reported by lintal (false positives)
    pub extra: Vec<Violation>,
}

impl TestMetrics {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            ..Default::default()
        }
    }

    /// Calculate compatibility percentage (detected / expected * 100)
    pub fn compatibility_percent(&self) -> f64 {
        let expected_count = self.detected.len() + self.missed.len();
        if expected_count == 0 {
            return 100.0;
        }
        (self.detected.len() as f64 / expected_count as f64) * 100.0
    }

    /// Calculate precision (detected / (detected + extra) * 100)
    pub fn precision_percent(&self) -> f64 {
        let reported_count = self.detected.len() + self.extra.len();
        if reported_count == 0 {
            return 100.0;
        }
        (self.detected.len() as f64 / reported_count as f64) * 100.0
    }

    /// Check if this test achieved 100% compatibility (no missed, no extra)
    pub fn is_perfect(&self) -> bool {
        self.missed.is_empty() && self.extra.is_empty()
    }

    /// Print a detailed report of the metrics
    pub fn print_report(&self) {
        println!("\n=== {} ===", self.test_name);
        println!(
            "Compatibility: {:.1}% ({}/{} expected)",
            self.compatibility_percent(),
            self.detected.len(),
            self.detected.len() + self.missed.len()
        );
        println!(
            "Precision: {:.1}% ({}/{} reported)",
            self.precision_percent(),
            self.detected.len(),
            self.detected.len() + self.extra.len()
        );

        if !self.missed.is_empty() {
            println!("\nMissed violations (false negatives):");
            for v in &self.missed {
                if let Some(name) = &v.name {
                    println!("  {}:{} - '{}'", v.line, v.column, name);
                } else {
                    println!("  {}:{}", v.line, v.column);
                }
            }
        }

        if !self.extra.is_empty() {
            println!("\nExtra violations (false positives):");
            for v in &self.extra {
                if let Some(name) = &v.name {
                    println!("  {}:{} - '{}'", v.line, v.column, name);
                } else {
                    println!("  {}:{}", v.line, v.column);
                }
            }
        }
    }
}

/// Aggregate metrics across multiple test cases for a rule.
#[derive(Debug, Clone, Default)]
pub struct RuleMetrics {
    pub rule_name: String,
    pub test_results: Vec<TestMetrics>,
}

impl RuleMetrics {
    pub fn new(rule_name: &str) -> Self {
        Self {
            rule_name: rule_name.to_string(),
            test_results: Vec::new(),
        }
    }

    pub fn add(&mut self, metrics: TestMetrics) {
        self.test_results.push(metrics);
    }

    /// Total detected across all tests
    pub fn total_detected(&self) -> usize {
        self.test_results.iter().map(|m| m.detected.len()).sum()
    }

    /// Total missed across all tests
    pub fn total_missed(&self) -> usize {
        self.test_results.iter().map(|m| m.missed.len()).sum()
    }

    /// Total extra across all tests
    pub fn total_extra(&self) -> usize {
        self.test_results.iter().map(|m| m.extra.len()).sum()
    }

    /// Overall compatibility percentage
    pub fn overall_compatibility(&self) -> f64 {
        let total_expected = self.total_detected() + self.total_missed();
        if total_expected == 0 {
            return 100.0;
        }
        (self.total_detected() as f64 / total_expected as f64) * 100.0
    }

    /// Overall precision percentage
    pub fn overall_precision(&self) -> f64 {
        let total_reported = self.total_detected() + self.total_extra();
        if total_reported == 0 {
            return 100.0;
        }
        (self.total_detected() as f64 / total_reported as f64) * 100.0
    }

    /// Check if all tests passed with 100% compatibility
    pub fn is_perfect(&self) -> bool {
        self.test_results.iter().all(|m| m.is_perfect())
    }

    /// Print aggregate report
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60));
        println!("RULE: {}", self.rule_name);
        println!("{}", "=".repeat(60));
        println!(
            "Overall Compatibility: {:.1}% ({}/{})",
            self.overall_compatibility(),
            self.total_detected(),
            self.total_detected() + self.total_missed()
        );
        println!(
            "Overall Precision: {:.1}% ({}/{})",
            self.overall_precision(),
            self.total_detected(),
            self.total_detected() + self.total_extra()
        );
        println!("Tests: {} total", self.test_results.len());

        let perfect_count = self.test_results.iter().filter(|m| m.is_perfect()).count();
        println!(
            "Perfect tests: {}/{} ({:.1}%)",
            perfect_count,
            self.test_results.len(),
            if self.test_results.is_empty() {
                100.0
            } else {
                (perfect_count as f64 / self.test_results.len() as f64) * 100.0
            }
        );

        // Show failing tests
        let failing: Vec<_> = self
            .test_results
            .iter()
            .filter(|m| !m.is_perfect())
            .collect();
        if !failing.is_empty() {
            println!("\nFailing tests:");
            for m in failing {
                println!(
                    "  {} - {:.1}% compat, {} missed, {} extra",
                    m.test_name,
                    m.compatibility_percent(),
                    m.missed.len(),
                    m.extra.len()
                );
            }
        }
    }
}

/// Parse expected violations from checkstyle test file comments.
///
/// Checkstyle marks expected violations with `// violation` comments.
/// Format:
/// - `// violation 'message'` - violation on this line
/// - `// violation above 'message'` - violation on previous line
/// - `// violation below 'message'` - violation on next line
/// - `// N violations M lines below:` - N violations M lines below
pub fn parse_expected_violations(source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Pattern for single violation comments
    // Matches: // violation 'Name 'foo' must match pattern'
    // Also: // violation above|below 'message'
    let single_violation_re =
        Regex::new(r"//\s*violation(?:\s+(above|below))?(?:\s+'[^']*')?").unwrap();

    // Pattern for multiple violations
    // Matches: // 2 violations 3 lines below:
    // Also: // 2 violations below
    // Also: // 2 violations (on same line)
    let multi_violation_re =
        Regex::new(r"//\s*(\d+)\s+violations?(?:\s+(?:(\d+)\s+lines?\s+)?(above|below))?").unwrap();

    let name_re = Regex::new(r"Name\s+'([^']+)'").unwrap();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        // Check for multi-violation pattern first (e.g., "// 2 violations 3 lines below:")
        if let Some(caps) = multi_violation_re.captures(line) {
            let count: usize = caps
                .get(1)
                .map(|m| m.as_str().parse().unwrap_or(1))
                .unwrap_or(1);
            let line_offset: usize = caps
                .get(2)
                .map(|m| m.as_str().parse().unwrap_or(1))
                .unwrap_or(1);
            let direction = caps.get(3).map(|m| m.as_str());

            let actual_line = match direction {
                Some("above") => line_num.saturating_sub(line_offset),
                Some("below") => line_num + line_offset,
                _ => line_num,
            };

            // Add the specified number of violations
            for _ in 0..count {
                violations.push(Violation {
                    line: actual_line,
                    column: 1,
                    name: None,
                });
            }
        } else if let Some(caps) = single_violation_re.captures(line) {
            // Single violation pattern
            let direction = caps.get(1).map(|m| m.as_str());
            let actual_line = match direction {
                Some("above") => line_num.saturating_sub(1),
                Some("below") => line_num + 1,
                _ => line_num,
            };

            // Try to extract the name from the message
            let name = name_re
                .captures(line)
                .map(|c| c.get(1).unwrap().as_str().to_string());

            violations.push(Violation {
                line: actual_line,
                column: 1,
                name,
            });
        }
    }

    violations
}

/// Parse config from checkstyle test file header comment.
///
/// Checkstyle test files have a header like:
/// ```java
/// /*
/// ConstantName
/// format = ^[A-Z][A-Z0-9]*(_[A-Z0-9]+)*$
/// applyToPublic = true
/// */
/// ```
pub fn parse_config_from_header(source: &str) -> Properties<'_> {
    let mut properties = Properties::new();

    // Find the header comment
    if let Some(start) = source.find("/*") {
        if let Some(end) = source[start..].find("*/") {
            let header = &source[start + 2..start + end];

            // Parse key = value lines
            for line in header.lines() {
                let line = line.trim();
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim();
                    let value = line[eq_pos + 1..].trim();
                    // Skip (default) values - let the rule use its actual default
                    // This avoids issues with Java-style escaping (e.g., \\. vs \.)
                    if value.starts_with("(default)") {
                        continue;
                    }
                    properties.insert(key, value);
                }
            }
        }
    }

    properties
}

/// Run a naming rule on source and collect violations.
pub fn run_rule<R: Rule + FromConfig>(source: &str, properties: &Properties) -> Vec<Violation> {
    let mut parser = JavaParser::new();
    let Some(result) = parser.parse(source) else {
        return vec![];
    };

    let rule = R::from_config(properties);
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
                name: None, // Could extract from diagnostic message
            });
        }
    }

    violations
}

/// Compare actual violations against expected and compute metrics.
pub fn compute_metrics(
    test_name: &str,
    actual: &[Violation],
    expected: &[Violation],
) -> TestMetrics {
    let mut metrics = TestMetrics::new(test_name);

    // Match violations by line (column matching is tricky due to different counting)
    for exp in expected {
        let matched = actual.iter().any(|a| a.line == exp.line);
        if matched {
            metrics.detected.push(exp.clone());
        } else {
            metrics.missed.push(exp.clone());
        }
    }

    for act in actual {
        let matched = expected.iter().any(|e| e.line == act.line);
        if !matched {
            metrics.extra.push(act.clone());
        }
    }

    metrics
}

/// Get path to checkstyle test repository.
fn checkstyle_repo_path() -> Option<PathBuf> {
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()? // lintal_linter -> crates
        .parent()? // crates -> lintal
        .join("target")
        .join("checkstyle-tests");

    if target_dir.join(".git").exists() {
        Some(target_dir)
    } else {
        None
    }
}

/// Load a checkstyle test input file for naming checks.
pub fn naming_test_input(check_name: &str, file_name: &str) -> Option<String> {
    let repo = checkstyle_repo_path()?;
    let path = repo
        .join("src/test/resources/com/puppycrawl/tools/checkstyle/checks/naming")
        .join(check_name.to_lowercase())
        .join(file_name);

    std::fs::read_to_string(&path).ok()
}

/// List all test input files for a naming check.
pub fn list_naming_test_files(check_name: &str) -> Vec<String> {
    let Some(repo) = checkstyle_repo_path() else {
        return vec![];
    };

    let dir: PathBuf = repo
        .join("src/test/resources/com/puppycrawl/tools/checkstyle/checks/naming")
        .join(check_name.to_lowercase());

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "java")
                .unwrap_or(false)
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

/// Run all tests for a naming rule and return aggregate metrics.
pub fn run_all_tests_for_rule<R: Rule + FromConfig>(
    rule_name: &str,
    check_name: &str,
) -> RuleMetrics {
    let mut rule_metrics = RuleMetrics::new(rule_name);

    for file_name in list_naming_test_files(check_name) {
        let Some(source) = naming_test_input(check_name, &file_name) else {
            continue;
        };

        let properties = parse_config_from_header(&source);
        let expected = parse_expected_violations(&source);
        let actual = run_rule::<R>(&source, &properties);

        let test_metrics = compute_metrics(&file_name, &actual, &expected);
        rule_metrics.add(test_metrics);
    }

    rule_metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expected_violations() {
        let source = r#"
class Foo {
    public static final int badConstant = 2; // violation 'Name 'badConstant' must match pattern'
    public static final int GOOD = 1;
    public static final int BAD__NAME = 3; // violation 'Name 'BAD__NAME' must match pattern'
}
"#;
        let violations = parse_expected_violations(source);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].name.as_deref(), Some("badConstant"));
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[1].name.as_deref(), Some("BAD__NAME"));
    }

    #[test]
    fn test_parse_config_from_header() {
        let source = r#"/*
ConstantName
format = ^[A-Z][A-Z0-9]*(_[A-Z0-9]+)*$
applyToPublic = (default)true
applyToPrivate = false
*/
package test;
"#;
        let config = parse_config_from_header(source);
        assert_eq!(config.get("format"), Some(&"^[A-Z][A-Z0-9]*(_[A-Z0-9]+)*$"));
        // (default) values are skipped so the rule uses its actual default
        assert_eq!(config.get("applyToPublic"), None);
        assert_eq!(config.get("applyToPrivate"), Some(&"false"));
    }

    #[test]
    fn test_compute_metrics() {
        let actual = vec![Violation::new(10, 5), Violation::new(20, 10)];
        let expected = vec![Violation::new(10, 5), Violation::new(30, 15)];

        let metrics = compute_metrics("test", &actual, &expected);
        assert_eq!(metrics.detected.len(), 1); // Line 10 matched
        assert_eq!(metrics.missed.len(), 1); // Line 30 missed
        assert_eq!(metrics.extra.len(), 1); // Line 20 extra
        assert!(!metrics.is_perfect());
        assert_eq!(metrics.compatibility_percent(), 50.0);
    }
}
