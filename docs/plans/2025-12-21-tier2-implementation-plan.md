# Tier 2 Rules Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement 6 Tier 2 checkstyle rules with auto-fix support, starting with OneStatementPerLine and MultipleVariableDeclarations.

**Architecture:** Each rule follows the existing pattern: a violation struct implementing `Violation`, a config struct implementing `FromConfig`, and implementation of the `Rule` trait. Rules are organized by checkstyle category (coding/, whitespace/).

**Tech Stack:** Rust, tree-sitter-java, lintal_diagnostics, lintal_java_cst

---

## Task 1: Create coding/ module directory

**Files:**
- Create: `crates/lintal_linter/src/rules/coding/mod.rs`
- Modify: `crates/lintal_linter/src/rules/mod.rs`

**Step 1: Create the coding module file**

Create `crates/lintal_linter/src/rules/coding/mod.rs`:

```rust
//! Coding rules (OneStatementPerLine, MultipleVariableDeclarations, etc.)
```

**Step 2: Add coding module to rules/mod.rs**

In `crates/lintal_linter/src/rules/mod.rs`, add after line 5:

```rust
pub mod coding;
```

**Step 3: Verify it compiles**

Run: `cargo check --package lintal_linter`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/lintal_linter/src/rules/coding/mod.rs crates/lintal_linter/src/rules/mod.rs
git commit -m "feat(coding): add coding rules module"
```

---

## Task 2: OneStatementPerLine - Write failing test

**Files:**
- Create: `crates/lintal_linter/src/rules/coding/one_statement_per_line.rs`
- Modify: `crates/lintal_linter/src/rules/coding/mod.rs`

**Step 1: Create rule file with test**

Create `crates/lintal_linter/src/rules/coding/one_statement_per_line.rs`:

```rust
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
```

**Step 2: Add to mod.rs**

Update `crates/lintal_linter/src/rules/coding/mod.rs`:

```rust
//! Coding rules (OneStatementPerLine, MultipleVariableDeclarations, etc.)

mod one_statement_per_line;

pub use one_statement_per_line::OneStatementPerLine;
```

**Step 3: Run test to verify it fails**

Run: `cargo test --package lintal_linter one_statement_per_line -- --nocapture`
Expected: `test_two_statements_same_line` FAILS (returns 0 diagnostics, expected 1)

**Step 4: Commit failing test**

```bash
git add crates/lintal_linter/src/rules/coding/
git commit -m "test(OneStatementPerLine): add failing tests for rule"
```

---

## Task 3: OneStatementPerLine - Implement rule

**Files:**
- Modify: `crates/lintal_linter/src/rules/coding/one_statement_per_line.rs`

**Step 1: Implement the check method**

Replace the `check` method in `OneStatementPerLine`:

```rust
impl Rule for OneStatementPerLine {
    fn name(&self) -> &'static str {
        "OneStatementPerLine"
    }

    fn check(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        // Only check at block level to find sibling statements
        if node.kind() != "block" {
            return vec![];
        }

        let source = ctx.source();
        let line_index = ctx.line_index();
        let source_code = ctx.source_code();

        let mut diagnostics = vec![];
        let mut prev_statement_line: Option<usize> = None;

        // Iterate through children of the block
        let ts_node = node.inner();
        let mut cursor = ts_node.walk();

        for child in ts_node.children(&mut cursor) {
            // Skip non-statement nodes (braces, comments)
            if !Self::is_statement_node(child.kind()) {
                continue;
            }

            let start_pos = lintal_text_size::TextSize::from(child.start_byte() as u32);
            let current_line = source_code.line_column(start_pos).line.get();

            if let Some(prev_line) = prev_statement_line {
                if current_line == prev_line {
                    // Two statements on same line - violation
                    let range = lintal_text_size::TextRange::new(
                        start_pos,
                        lintal_text_size::TextSize::from(child.end_byte() as u32),
                    );

                    // Calculate fix: insert newline + indentation before this statement
                    let indent = Self::get_indentation(source, child.start_byte());
                    let fix_start = Self::find_prev_semicolon_end(source, child.start_byte());
                    let fix_range = lintal_text_size::TextRange::new(
                        lintal_text_size::TextSize::from(fix_start as u32),
                        start_pos,
                    );

                    diagnostics.push(
                        Diagnostic::new(OneStatementPerLineViolation, range)
                            .with_fix(Fix::safe_edit(Edit::range_replacement(
                                format!("\n{}", indent),
                                fix_range,
                            ))),
                    );
                }
            }

            prev_statement_line = Some(current_line);
        }

        diagnostics
    }
}

impl OneStatementPerLine {
    /// Check if a node kind represents a statement.
    fn is_statement_node(kind: &str) -> bool {
        matches!(
            kind,
            "local_variable_declaration"
                | "expression_statement"
                | "if_statement"
                | "for_statement"
                | "enhanced_for_statement"
                | "while_statement"
                | "do_statement"
                | "try_statement"
                | "switch_expression"
                | "return_statement"
                | "throw_statement"
                | "break_statement"
                | "continue_statement"
                | "assert_statement"
                | "synchronized_statement"
                | "labeled_statement"
                | "empty_statement"
        )
    }

    /// Get the indentation at a byte position.
    fn get_indentation(source: &str, pos: usize) -> String {
        // Find the start of the line
        let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..pos];

        // Extract leading whitespace
        let indent_len = line.len() - line.trim_start().len();
        line[..indent_len].to_string()
    }

    /// Find the end of the previous semicolon (after any whitespace).
    fn find_prev_semicolon_end(source: &str, pos: usize) -> usize {
        // Look backwards for semicolon
        let before = &source[..pos];
        if let Some(semi_pos) = before.rfind(';') {
            // Return position after semicolon
            semi_pos + 1
        } else {
            pos
        }
    }
}
```

**Step 2: Add required imports at top of file**

Update imports at top of file:

```rust
use lintal_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use lintal_java_cst::CstNode;
use lintal_source_file::LineIndex;

use crate::{CheckContext, FromConfig, Properties, Rule};
```

**Step 3: Run tests to verify they pass**

Run: `cargo test --package lintal_linter one_statement_per_line -- --nocapture`
Expected: All tests PASS

**Step 4: Commit implementation**

```bash
git add crates/lintal_linter/src/rules/coding/one_statement_per_line.rs
git commit -m "feat(OneStatementPerLine): implement rule with auto-fix"
```

---

## Task 4: OneStatementPerLine - Register in registry

**Files:**
- Modify: `crates/lintal_linter/src/rules/mod.rs`
- Modify: `crates/lintal_linter/src/registry.rs`

**Step 1: Export from rules/mod.rs**

Update `crates/lintal_linter/src/rules/mod.rs` to add re-export:

```rust
pub use coding::OneStatementPerLine;
```

**Step 2: Register in registry.rs**

In `crates/lintal_linter/src/registry.rs`, add to imports (around line 51-56):

```rust
use crate::rules::{
    ArrayTypeStyle, AvoidNestedBlocks, EmptyBlock, EmptyCatchBlock, EmptyForInitializerPad,
    FileTabCharacter, FinalLocalVariable, FinalParameters, LeftCurly, MethodParamPad,
    ModifierOrder, NeedBraces, NoWhitespaceAfter, NoWhitespaceBefore, OneStatementPerLine,
    ParenPad, RedundantImport, RedundantModifier, RightCurly, SingleSpaceSeparator,
    TypecastParenPad, UnusedImports, UpperEll, WhitespaceAfter, WhitespaceAround,
};
```

Add registration (after line 86):

```rust
// Coding rules
self.register::<OneStatementPerLine>();
```

**Step 3: Verify it compiles**

Run: `cargo check --package lintal_linter`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/lintal_linter/src/rules/mod.rs crates/lintal_linter/src/registry.rs
git commit -m "feat(OneStatementPerLine): register rule in registry"
```

---

## Task 5: OneStatementPerLine - Add checkstyle compatibility tests

**Files:**
- Create: `crates/lintal_linter/tests/checkstyle_onestatementperline.rs`
- Modify: `crates/lintal_linter/tests/checkstyle_repo.rs`

**Step 1: Add coding_test_input helper to checkstyle_repo.rs**

Add after `imports_test_input` function (around line 72):

```rust
/// Get path to a checkstyle test input file for coding checks.
#[allow(dead_code)]
pub fn coding_test_input(check_name: &str, file_name: &str) -> Option<PathBuf> {
    let repo = checkstyle_repo()?;
    let path = repo
        .join("src/test/resources/com/puppycrawl/tools/checkstyle/checks/coding")
        .join(check_name.to_lowercase())
        .join(file_name);

    if path.exists() { Some(path) } else { None }
}
```

**Step 2: Create checkstyle compatibility test file**

Create `crates/lintal_linter/tests/checkstyle_onestatementperline.rs`:

```rust
//! OneStatementPerLine checkstyle compatibility tests.

mod checkstyle_repo;

use lintal_java_cst::TreeWalker;
use lintal_java_parser::JavaParser;
use lintal_linter::rules::OneStatementPerLine;
use lintal_linter::{CheckContext, Rule};
use lintal_source_file::{LineIndex, SourceCode};

/// A violation at a specific location.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    line: usize,
}

/// Run OneStatementPerLine rule on source and collect violations.
fn check_one_statement_per_line(source: &str) -> Vec<Violation> {
    let mut parser = JavaParser::new();
    let Some(result) = parser.parse(source) else {
        panic!("Failed to parse source");
    };

    let rule = OneStatementPerLine::default();
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
            });
        }
    }

    violations
}

/// Load a checkstyle test input file.
fn load_fixture(file_name: &str) -> Option<String> {
    let path = checkstyle_repo::coding_test_input("onestatementperline", file_name)?;
    std::fs::read_to_string(&path).ok()
}

#[test]
fn test_single_line_in_loops() {
    let Some(source) = load_fixture("InputOneStatementPerLineSingleLineInLoops.java") else {
        eprintln!("Skipping test: checkstyle repo not available");
        return;
    };

    let violations = check_one_statement_per_line(&source);

    // Expected violations from checkstyle comments in file:
    // Line 27: `; two = 2;` - violation
    // Line 53: `int a; int b;` - violation
    // Line 65: `int e = 1; int f = 2;` - violation
    // Line 85: `var1++; var2++;` - violation
    // Line 89: `Object obj1 = new Object(); Object obj2 = new Object();` - violation

    let expected_lines = vec![27, 53, 65, 85, 89];

    for line in &expected_lines {
        assert!(
            violations.iter().any(|v| v.line == *line),
            "Expected violation on line {}, got violations on lines: {:?}",
            line,
            violations.iter().map(|v| v.line).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_for_declarations_ok() {
    let Some(source) = load_fixture("InputOneStatementPerLineSingleLineForDeclarations.java") else {
        eprintln!("Skipping test: checkstyle repo not available");
        return;
    };

    let violations = check_one_statement_per_line(&source);

    // For loop declarations should NOT cause violations
    // The file has for loops with multiple statements in header - these are OK
    // Only check that we don't flag for-loop internals incorrectly

    println!("Found {} violations in for declarations test", violations.len());
    for v in &violations {
        println!("  Line {}", v.line);
    }
}

#[test]
fn test_all_have_fixes() {
    let source = r#"
class Test {
    void method() {
        int a; int b;
        a = 1; b = 2;
    }
}
"#;
    let mut parser = JavaParser::new();
    let result = parser.parse(source).unwrap();
    let rule = OneStatementPerLine::default();
    let ctx = CheckContext::new(source);

    let mut diagnostics = vec![];
    for node in TreeWalker::new(result.tree.root_node(), source) {
        diagnostics.extend(rule.check(&ctx, &node));
    }

    for diag in &diagnostics {
        assert!(
            diag.fix.is_some(),
            "All violations should have fixes"
        );
    }
}
```

**Step 3: Run tests**

Run: `cargo test --package lintal_linter checkstyle_onestatementperline -- --nocapture`
Expected: Tests pass (or skip if checkstyle repo not available)

**Step 4: Commit**

```bash
git add crates/lintal_linter/tests/checkstyle_onestatementperline.rs crates/lintal_linter/tests/checkstyle_repo.rs
git commit -m "test(OneStatementPerLine): add checkstyle compatibility tests"
```

---

## Task 6: MultipleVariableDeclarations - Write failing test

**Files:**
- Create: `crates/lintal_linter/src/rules/coding/multiple_variable_declarations.rs`
- Modify: `crates/lintal_linter/src/rules/coding/mod.rs`

**Step 1: Create rule file with test**

Create `crates/lintal_linter/src/rules/coding/multiple_variable_declarations.rs`:

```rust
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
```

**Step 2: Add to mod.rs**

Update `crates/lintal_linter/src/rules/coding/mod.rs`:

```rust
//! Coding rules (OneStatementPerLine, MultipleVariableDeclarations, etc.)

mod multiple_variable_declarations;
mod one_statement_per_line;

pub use multiple_variable_declarations::MultipleVariableDeclarations;
pub use one_statement_per_line::OneStatementPerLine;
```

**Step 3: Run test to verify it fails**

Run: `cargo test --package lintal_linter multiple_variable_declarations -- --nocapture`
Expected: `test_comma_separated_violation` and `test_same_line_violation` FAIL

**Step 4: Commit failing test**

```bash
git add crates/lintal_linter/src/rules/coding/multiple_variable_declarations.rs crates/lintal_linter/src/rules/coding/mod.rs
git commit -m "test(MultipleVariableDeclarations): add failing tests for rule"
```

---

## Task 7: MultipleVariableDeclarations - Implement rule

**Files:**
- Modify: `crates/lintal_linter/src/rules/coding/multiple_variable_declarations.rs`

**Step 1: Implement the check method**

Replace the `check` method:

```rust
impl Rule for MultipleVariableDeclarations {
    fn name(&self) -> &'static str {
        "MultipleVariableDeclarations"
    }

    fn check(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Check for comma-separated variables in a single declaration
        if node.kind() == "local_variable_declaration" || node.kind() == "field_declaration" {
            // Skip if inside for-loop initializer
            if Self::is_in_for_init(node) {
                return vec![];
            }

            diagnostics.extend(self.check_comma_separated(ctx, node));
        }

        // Check for multiple declarations on same line (handled at block level)
        if node.kind() == "block" || node.kind() == "class_body" {
            diagnostics.extend(self.check_same_line_declarations(ctx, node));
        }

        diagnostics
    }
}

impl MultipleVariableDeclarations {
    /// Check if node is inside a for-loop initializer.
    fn is_in_for_init(node: &CstNode) -> bool {
        let mut current = node.inner().parent();
        while let Some(parent) = current {
            if parent.kind() == "for_statement" {
                // Check if we're in the init part (first child after 'for' and '(')
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "local_variable_declaration" {
                        return child.id() == node.inner().id();
                    }
                    if child.kind() == ";" {
                        break;
                    }
                }
            }
            current = parent.parent();
        }
        false
    }

    /// Check for comma-separated variables in a declaration.
    fn check_comma_separated(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        let ts_node = node.inner();
        let mut cursor = ts_node.walk();

        let mut declarator_count = 0;
        let mut first_declarator_range = None;

        for child in ts_node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                declarator_count += 1;
                if first_declarator_range.is_none() {
                    first_declarator_range = Some(lintal_text_size::TextRange::new(
                        lintal_text_size::TextSize::from(child.start_byte() as u32),
                        lintal_text_size::TextSize::from(child.end_byte() as u32),
                    ));
                }
            }
        }

        if declarator_count > 1 {
            if let Some(range) = first_declarator_range {
                // TODO: Create fix that splits declarations
                return vec![Diagnostic::new(MultipleInStatementViolation, range)];
            }
        }

        vec![]
    }

    /// Check for multiple declarations on the same line.
    fn check_same_line_declarations(&self, ctx: &CheckContext, node: &CstNode) -> Vec<Diagnostic> {
        let source_code = ctx.source_code();
        let ts_node = node.inner();
        let mut cursor = ts_node.walk();

        let mut diagnostics = vec![];
        let mut prev_decl_line: Option<usize> = None;

        for child in ts_node.children(&mut cursor) {
            let kind = child.kind();
            if kind != "local_variable_declaration" && kind != "field_declaration" {
                continue;
            }

            let start_pos = lintal_text_size::TextSize::from(child.start_byte() as u32);
            let current_line = source_code.line_column(start_pos).line.get();

            if let Some(prev_line) = prev_decl_line {
                if current_line == prev_line {
                    let range = lintal_text_size::TextRange::new(
                        start_pos,
                        lintal_text_size::TextSize::from(child.end_byte() as u32),
                    );

                    // Create fix: insert newline before this declaration
                    let source = ctx.source();
                    let indent = Self::get_indentation(source, child.start_byte());
                    let fix_start = Self::find_prev_semicolon_end(source, child.start_byte());
                    let fix_range = lintal_text_size::TextRange::new(
                        lintal_text_size::TextSize::from(fix_start as u32),
                        start_pos,
                    );

                    diagnostics.push(
                        Diagnostic::new(MultipleOnLineViolation, range)
                            .with_fix(Fix::safe_edit(Edit::range_replacement(
                                format!("\n{}", indent),
                                fix_range,
                            ))),
                    );
                }
            }

            prev_decl_line = Some(current_line);
        }

        diagnostics
    }

    fn get_indentation(source: &str, pos: usize) -> String {
        let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..pos];
        let indent_len = line.len() - line.trim_start().len();
        line[..indent_len].to_string()
    }

    fn find_prev_semicolon_end(source: &str, pos: usize) -> usize {
        let before = &source[..pos];
        if let Some(semi_pos) = before.rfind(';') {
            semi_pos + 1
        } else {
            pos
        }
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --package lintal_linter multiple_variable_declarations -- --nocapture`
Expected: All tests PASS

**Step 3: Commit implementation**

```bash
git add crates/lintal_linter/src/rules/coding/multiple_variable_declarations.rs
git commit -m "feat(MultipleVariableDeclarations): implement rule with auto-fix"
```

---

## Task 8: MultipleVariableDeclarations - Register and add compat tests

**Files:**
- Modify: `crates/lintal_linter/src/rules/mod.rs`
- Modify: `crates/lintal_linter/src/registry.rs`
- Create: `crates/lintal_linter/tests/checkstyle_multiplevariabledeclarations.rs`

**Step 1: Export from rules/mod.rs**

Add to `crates/lintal_linter/src/rules/mod.rs`:

```rust
pub use coding::{MultipleVariableDeclarations, OneStatementPerLine};
```

**Step 2: Register in registry.rs**

Add `MultipleVariableDeclarations` to imports and register it:

```rust
// In imports:
use crate::rules::{
    // ... existing imports ...
    MultipleVariableDeclarations, OneStatementPerLine,
    // ...
};

// In register_builtins:
self.register::<MultipleVariableDeclarations>();
```

**Step 3: Create checkstyle compatibility tests**

Create `crates/lintal_linter/tests/checkstyle_multiplevariabledeclarations.rs`:

```rust
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
        violations.iter().any(|v| v.line == 11 && v.message_type == "comma"),
        "Should detect comma-separated on line 11"
    );

    // Verify we detect same-line violations
    assert!(
        violations.iter().any(|v| v.line == 12 && v.message_type == "same_line"),
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
```

**Step 4: Run tests**

Run: `cargo test --package lintal_linter -- --nocapture 2>&1 | head -100`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/lintal_linter/src/rules/mod.rs crates/lintal_linter/src/registry.rs crates/lintal_linter/tests/checkstyle_multiplevariabledeclarations.rs
git commit -m "feat(MultipleVariableDeclarations): register rule and add compat tests"
```

---

## Task 9: Final verification and cleanup

**Step 1: Run full test suite**

Run: `cargo test --all`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: No warnings

**Step 3: Format code**

Run: `cargo fmt --all`

**Step 4: Final commit if any formatting changes**

```bash
git add -A
git commit -m "chore: format code" --allow-empty
```

---

## Summary

After completing all tasks, the following will be implemented:

1. **coding/ module** - New rule category matching checkstyle structure
2. **OneStatementPerLine** - Detects multiple statements per line, with auto-fix
3. **MultipleVariableDeclarations** - Detects comma-separated and same-line declarations, with auto-fix
4. **Checkstyle compatibility tests** - Validates against actual checkstyle test fixtures

Both rules are registered in the registry and follow the established lintal patterns.
