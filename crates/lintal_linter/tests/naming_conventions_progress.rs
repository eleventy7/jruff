//! Aggregate progress tracking for naming convention rules.
//!
//! This test provides a summary view of checkstyle compatibility across
//! all naming convention rules. Run with:
//!
//! ```
//! cargo test --package lintal_linter --test naming_conventions_progress -- --nocapture
//! ```

mod naming_test_utils;

use naming_test_utils::RuleMetrics;

/// Summary of all naming convention rules
#[derive(Debug, Default)]
pub struct NamingConventionsProgress {
    pub rules: Vec<RuleMetrics>,
}

impl NamingConventionsProgress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, rule: RuleMetrics) {
        self.rules.push(rule);
    }

    pub fn total_detected(&self) -> usize {
        self.rules.iter().map(|r| r.total_detected()).sum()
    }

    pub fn total_missed(&self) -> usize {
        self.rules.iter().map(|r| r.total_missed()).sum()
    }

    pub fn total_extra(&self) -> usize {
        self.rules.iter().map(|r| r.total_extra()).sum()
    }

    pub fn overall_compatibility(&self) -> f64 {
        let total_expected = self.total_detected() + self.total_missed();
        if total_expected == 0 {
            return 100.0;
        }
        (self.total_detected() as f64 / total_expected as f64) * 100.0
    }

    pub fn overall_precision(&self) -> f64 {
        let total_reported = self.total_detected() + self.total_extra();
        if total_reported == 0 {
            return 100.0;
        }
        (self.total_detected() as f64 / total_reported as f64) * 100.0
    }

    pub fn print_summary(&self) {
        println!("\n");
        println!("╔════════════════════════════════════════════════════════════════════╗");
        println!("║            NAMING CONVENTIONS - CHECKSTYLE COMPATIBILITY           ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");

        // Per-rule summary
        println!(
            "║ {:24} {:>10} {:>10} {:>10} {:>8} ║",
            "Rule", "Detected", "Missed", "Extra", "Compat%"
        );
        println!("╠════════════════════════════════════════════════════════════════════╣");

        for rule in &self.rules {
            let compat = rule.overall_compatibility();
            let has_tests = rule.total_detected() > 0 || rule.total_missed() > 0;
            let status = if !has_tests {
                "·" // Not implemented
            } else if rule.is_perfect() {
                "✓"
            } else if compat >= 90.0 {
                "~"
            } else {
                "○"
            };

            println!(
                "║ {} {:22} {:>10} {:>10} {:>10} {:>7.1}% ║",
                status,
                rule.rule_name,
                rule.total_detected(),
                rule.total_missed(),
                rule.total_extra(),
                compat
            );
        }

        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!(
            "║   {:22} {:>10} {:>10} {:>10} {:>7.1}% ║",
            "TOTAL",
            self.total_detected(),
            self.total_missed(),
            self.total_extra(),
            self.overall_compatibility()
        );
        println!("╚════════════════════════════════════════════════════════════════════╝");

        // Legend
        println!("\nLegend: ✓ = 100%  ~ = ≥90%  ○ = partial  · = not implemented");

        // Show target
        let implemented = self
            .rules
            .iter()
            .filter(|r| r.total_detected() > 0 || r.total_missed() > 0)
            .count();
        println!("\nImplemented: {}/{} rules", implemented, self.rules.len());

        let total_expected = self.total_detected() + self.total_missed();
        if total_expected > 0 && self.overall_compatibility() == 100.0 && self.total_extra() == 0 {
            println!("\n🎉 100% CHECKSTYLE COMPATIBILITY ACHIEVED! 🎉");
        }
    }
}

/// Main progress tracking test.
///
/// This test runs all naming convention rules against checkstyle's test suite
/// and reports aggregate compatibility metrics.
#[test]
fn test_naming_conventions_progress() {
    let mut progress = NamingConventionsProgress::new();

    // Add metrics for each naming rule (even if not implemented yet)
    // Rules are added in order of planned implementation

    // ConstantName - first to implement
    let constant_name = collect_constant_name_metrics();
    progress.add(constant_name);

    // TypeName - second to implement
    let type_name = collect_type_name_metrics();
    progress.add(type_name);

    // MethodName
    let method_name = collect_method_name_metrics();
    progress.add(method_name);

    // MemberName
    let member_name = collect_member_name_metrics();
    progress.add(member_name);

    // ParameterName
    let parameter_name = collect_parameter_name_metrics();
    progress.add(parameter_name);

    // LocalVariableName
    let local_variable_name = collect_local_variable_name_metrics();
    progress.add(local_variable_name);

    // LocalFinalVariableName
    let local_final_variable_name = collect_local_final_variable_name_metrics();
    progress.add(local_final_variable_name);

    // StaticVariableName
    let static_variable_name = collect_static_variable_name_metrics();
    progress.add(static_variable_name);

    // PackageName
    let package_name = collect_package_name_metrics();
    progress.add(package_name);

    // Print the summary
    progress.print_summary();

    // This test always passes - it's for reporting progress
    // We'll add assertions once we're targeting 100%
}

// Metric collection functions for each rule

fn collect_constant_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::ConstantName;
    naming_test_utils::run_all_tests_for_rule::<ConstantName>("ConstantName", "constantname")
}

fn collect_type_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::TypeName;
    naming_test_utils::run_all_tests_for_rule::<TypeName>("TypeName", "typename")
}

fn collect_method_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::MethodName;
    naming_test_utils::run_all_tests_for_rule::<MethodName>("MethodName", "methodname")
}

fn collect_member_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::MemberName;
    naming_test_utils::run_all_tests_for_rule::<MemberName>("MemberName", "membername")
}

fn collect_parameter_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::ParameterName;
    naming_test_utils::run_all_tests_for_rule::<ParameterName>("ParameterName", "parametername")
}

fn collect_local_variable_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::LocalVariableName;
    naming_test_utils::run_all_tests_for_rule::<LocalVariableName>(
        "LocalVariableName",
        "localvariablename",
    )
}

fn collect_local_final_variable_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::LocalFinalVariableName;
    let metrics = naming_test_utils::run_all_tests_for_rule::<LocalFinalVariableName>(
        "LocalFinalVariableName",
        "localfinalvariablename",
    );

    for test in &metrics.test_results {
        if !test.missed.is_empty() || !test.extra.is_empty() {
            eprintln!(
                "\n[DEBUG] {}: {} missed, {} extra",
                test.test_name,
                test.missed.len(),
                test.extra.len()
            );
            for v in &test.missed {
                eprintln!("  MISSED: Line {}", v.line);
            }
            for v in &test.extra {
                eprintln!("  EXTRA: Line {}", v.line);
            }
        }
    }

    metrics
}

fn collect_static_variable_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::StaticVariableName;
    naming_test_utils::run_all_tests_for_rule::<StaticVariableName>(
        "StaticVariableName",
        "staticvariablename",
    )
}

fn collect_package_name_metrics() -> RuleMetrics {
    use lintal_linter::rules::PackageName;
    let metrics =
        naming_test_utils::run_all_tests_for_rule::<PackageName>("PackageName", "packagename");

    for test in &metrics.test_results {
        if !test.missed.is_empty() || !test.extra.is_empty() {
            eprintln!(
                "\n[DEBUG] {}: {} missed, {} extra",
                test.test_name,
                test.missed.len(),
                test.extra.len()
            );
            for v in &test.missed {
                eprintln!("  MISSED: Line {}", v.line);
            }
            for v in &test.extra {
                eprintln!("  EXTRA: Line {}", v.line);
            }
        }
    }

    metrics
}
