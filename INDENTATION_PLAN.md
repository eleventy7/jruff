# Indentation Rule Improvement Plan

**Current Status:** 83.7% detection rate (139 missing, 20 extra)
**Exact Matches:** 125/174 files (71.8%)
**Goal:** 100% - exact match on all 174 test fixtures

## Recent Fixes

The following lenient mode issues have been fixed:
- Use base indent for lenient checking in binary expressions
- Pass base argument indent for nested method invocations
- Use new_indent for type/lparen continuation checks
- Use lenient check for method name continuation
- **Anonymous class body indent**: Accept multiple levels including `new + basicOffset` for over-indented anonymous classes in method arguments (fixes StrictCondition lines 9, 11)

## Remaining Extra Violations (20)

| File | Extra | Issue |
|------|-------|-------|
| InputIndentationMembers.java | 1 | Nested method call argument indent (line 54, `exp:>=14`) |
| InputIndentationCodeBlocks2.java | 1 | Unknown |
| InputIndentationAnnotationArrayInitOldStyle.java | 1 | Annotation array init |
| InputIndentationTryWithResourcesStrict.java | 1 | Try-with-resources |
| InputIndentationTryResourcesNotStrict1.java | 4 | Try-with-resources lenient mode |
| InputIndentationTextBlock.java | 3 | Text block handling |
| Various DIFF files | ~9 | Mixed missing/extra issues |

**Pattern to look for in test files:** `exp:>=N` means lenient mode (accept N or higher).

### Members.java Line 54 Issue

The nested method call argument at line 54 (`new SecondFieldClassWithVeryVeryVeryLongName`) is at indent 19, expected `>=14`. We check against `combined_arg_indent` (20), but in lenient mode should accept >= 14. Attempted fix with `lenient_arg_indent = combined_arg_indent.combine(indent)` caused regressions elsewhere - needs more targeted approach.

---

### 2. HIGH IMPACT - forceStrictCondition Support (11 missing)

**File:** InputIndentationNewWithForceStrictCondition.java

**Config:** `forceStrictCondition=true`, `lineWrappingIndentation=8`

**Missing patterns:**
- Line 21: Array bracket `]` continuation (11 vs 12)
- Line 25: Array bracket `[]` at wrong position (4 vs 12)
- Line 31: Nested `new` inside another `new` argument (16 vs 24)
- Line 32: Anonymous class body content (20 vs 28,32,36)
- Line 33: Closing brace/paren (16 vs 24,28,32)
- Line 35: Binary expression continuation (35 vs 16)

**Debug command:**
```bash
# Add this test first:
# fn test_debug_force_strict() { debug_fixture("InputIndentationNewWithForceStrictCondition.java"); }
cargo test --package lintal_linter --test checkstyle_indentation test_debug_force_strict -- --nocapture
```

**AST dump:**
```bash
cat /Users/shaunlaurens/src/lintal/target/checkstyle-tests/src/test/resources/com/puppycrawl/tools/checkstyle/checks/indentation/indentation/InputIndentationNewWithForceStrictCondition.java | head -40 | ./target/debug/dump_java_ast
```

**Fix needed:** Add config override for this file in `get_config_overrides()` function, then fix the specific patterns.

---

### 3. MEDIUM IMPACT - Catch Parameters (5 missing)

**File:** InputIndentationCatchParametersOnNewLine.java

**Missing patterns:**
- Multi-catch `|` separator on new line
- Exception type continuation after annotation
- Annotation before exception type

**Debug command:**
```bash
# Add: fn test_debug_catch_params() { debug_fixture("InputIndentationCatchParametersOnNewLine.java"); }
```

**AST structure:**
```
catch_clause
  catch_formal_parameter
    modifiers (may contain annotations)
    catch_type
      type_identifier
      | (for multi-catch)
      type_identifier
```

**Fix location:** `check_try_statement` in mod.rs - need to add catch parameter checks.

---

### 4. MEDIUM IMPACT - Anonymous Class Curly on New Line (6 missing)

**File:** InputIndentationAnonymousClassInMethodCurlyOnNewLine.java

**Pattern:** When anonymous class `{` is on a new line, checkstyle expects specific indent levels.

Expected format: `exp:16,20,24 warn` means any of those values is expected, but actual is wrong.

**Debug command:**
```bash
# Add: fn test_debug_anon_class_curly() { debug_fixture("InputIndentationAnonymousClassInMethodCurlyOnNewLine.java"); }
```

**Fix location:** `check_object_creation_expression` - handle class_body opening brace on new line.

---

### 5. MEDIUM IMPACT - Annotation Closing Paren (5 missing)

**File:** InputIndentationAnnotationClosingParenthesisEndsInSameIndentationAsOpen.java

**Pattern:** Annotation `)` on new line should match `@` or `(` indent.

```java
@SimpleType( value = Boolean.class
                )   // indent:16 exp:0 warn - should match @SimpleType
```

**Fix location:** `check_modifiers_annotations` - add check for rparen continuation.

---

### 6. LOWER IMPACT - Various Patterns

| File | Missing | Pattern |
|------|---------|---------|
| InputIndentationInvalidClassDefIndent1.java | 9 | Class def continuation |
| InputIndentationLineWrappedRecordDeclaration.java | 6 | Record declarations |
| InputIndentationInvalidWhileIndent.java | 6 | While loop conditions |
| InputIndentationInvalidDoWhileIndent.java | 5 | Do-while conditions |
| InputIndentationRecordsAndCompactCtors.java | 3 | Record compact constructors |
| InputIndentationInvalidTryIndent.java | 3 | Try statement parts |
| InputIndentationInvalidSwitchIndent.java | 3 | Switch cases |
| InputIndentationTextBlock.java | 3 | Text blocks |

---

## Implementation Order

### Day 1: Fix False Positives First (Extra violations)

1. **Fix lenient mode in object creation args** (mod.rs:2479-2506)
   - Currently using strict `is_acceptable()`
   - Need hybrid: strict for over-indent, lenient for under-indent
   - Test: `test_debug_members`, `test_debug_chained_method_calls`

2. **Fix chained method call false positives**
   - Lines 43-45 in InputIndentationChainedMethodCalls.java flagged incorrectly
   - They're at correct indent but being flagged

### Day 2: Add Missing Constructs

3. **Catch parameters** - new check needed
4. **Annotation closing paren** - extend `check_modifiers_annotations`
5. **Anonymous class brace on new line** - extend `check_object_creation_expression`

### Day 3: Special Configs and Edge Cases

6. **forceStrictCondition=true files** - add config overrides
7. **Record declarations** - may need new handlers
8. **Text blocks** - special string literal handling

---

## Quick Start Commands

```bash
# Run compatibility summary
cargo test --package lintal_linter --test checkstyle_indentation test_fixture_compatibility_summary -- --nocapture

# Debug specific file (add test function first if needed)
cargo test --package lintal_linter --test checkstyle_indentation test_debug_members -- --nocapture

# Dump AST for investigation
cat /path/to/file.java | ./target/debug/dump_java_ast

# Build release for performance testing
cargo build --release
```

## Key Files

- **Rule implementation:** `crates/lintal_linter/src/rules/whitespace/indentation/mod.rs`
- **Tests:** `crates/lintal_linter/tests/checkstyle_indentation.rs`
- **IndentLevel helper:** `crates/lintal_linter/src/rules/whitespace/indentation/indent_level.rs`
- **Test fixtures:** `target/checkstyle-tests/src/test/resources/com/puppycrawl/tools/checkstyle/checks/indentation/indentation/`

## Debug Test Template

Add to `checkstyle_indentation.rs`:
```rust
#[test]
fn test_debug_YOUR_TEST() {
    debug_fixture("InputIndentationYOUR_FILE.java");
}
```

## Config Override Template

Add to `get_config_overrides()` in test file:
```rust
"InputIndentationNewWithForceStrictCondition.java" => Some([
    ("forceStrictCondition", "true"),
    ("lineWrappingIndentation", "8"),
    // ... other config
].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()),
```
