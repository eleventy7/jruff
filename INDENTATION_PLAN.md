# Indentation Rule Improvement Plan

**Current Status:** 92.7% detection rate (62 missing, 0 extra)
**Exact Matches:** 143/174 files (82.2%)
**Goal:** 100% - exact match on all 174 test fixtures

## Recent Fixes (Session Dec 30 - Continued)

### Method Call and Constructor Argument Alignment Fixes

1. **Return statement context**: Skip argument indentation check for method calls in return statements
   - Checkstyle accepts ANY indentation for method call args in return statements
   - Added `in_return_context` check for both method_invocation and object_creation_expression

2. **Field declaration context**: Skip argument indentation check for method calls in field initializers
   - Similar lenient behavior to return statements

3. **Constructor args alignment**: Accept alignment with `new` position
   - For `new Constructor(arg1, arg2,\n            arg3)`, args can be at `new` column

4. **Nested method call args**: Accept alignment with outer context
   - For `assertEquals(\n    foo(bar(\n    VALUE)))`, VALUE can align with outer arg level

5. **Annotation array initializers**: Use attribute line indent as base
   - For `@Ann(names = { "A", "B" })`, elements indent from attribute line, not class level

**Real-world results after fix:**
| Project | Session Start | After Lambda Fix | After All Fixes | Total Reduction |
|---------|---------------|------------------|-----------------|-----------------|
| agrona  | 79            | 18               | 17              | 78%             |
| artio   | 1813          | 275              | 0               | 100%            |
| aeron   | 160           | 161              | 37              | 77%             |
| **Total** | **2052**    | **454**          | **54**          | **97%**         |

**Test suite:** 92.7% detection (62 missing, 0 extra), 143/174 exact matches (82.2%)

### Tests Added
- `test_return_statement_args_any_indent` - codifies return statement leniency
- `test_field_declaration_args_lenient` - codifies field declaration leniency
- `test_expression_statement_args_strict` - verifies expression statements still checked
- Updated real-world pattern tests to use lenient mode

## Previous Session (Dec 30)

### Lambda Block at Statement Level Fix
- **Key Fix**: When lambda block brace appears on a NEW LINE and lambda is at statement level
  (not on a continuation line), accept the brace at the statement level
- Added `lambda_at_statement_level` check to avoid over-leniency for continuation lambdas

**Pattern fixed:**
```java
executor.submit(() ->
{              // Now accepted at statement level (col 8)
    doWork();  // Content at col 12 (8 + 4)
});
```

**Pattern correctly NOT changed:**
```java
Function<String, String> f =
        (string) -> {   // Lambda at continuation (col 16)
            work();     // Content at 20 is still flagged (expected 16)
        };
```

## Recent Fixes (Session Dec 29 - Continued pt9)

### Real-World Project Testing
- **artio**: 0 checkstyle violations, 1813 lintal false positives
- **agrona**: 0 checkstyle violations, 79 lintal false positives
- **aeron**: 0 checkstyle violations, 160 lintal false positives

Main false positive patterns identified:
- Lambda blocks in method call arguments (dominant issue) - **FIXED**
- `new` expressions with complex nesting
- Annotation array initializers

## Recent Fixes (Session Dec 29 - Continued pt8)

### Array Initialization Context Fixes
- **InputIndentationArrays.java**: Fixed 8 extra violations for `return new byte[] {` case
- **InputIndentationValidArrayInitIndentTwo.java**: Fixed 8 extra violations for field init case
- **InputIndentationNewHandler.java**: Fixed 2 extra violations for nested array_creation_expression

**Key changes:**
- Added `check_array_creation_expression_with_context` to distinguish variable init vs expression context
- Variable initializers (`int[] x = new int[] {...}`) use `arrayInitIndent` for elements
- Expression contexts (`return new byte[] {...}`) use `lineWrappingIndentation` for elements
- For inline brace with content on same line, also accept alignment with first element
- For misaligned parent braces, use lenient mode only for nested `array_creation_expression` children

**Files changed:**
- `mod.rs`: Added context-aware array creation handling
- Variable init path now calls `check_array_creation_expression_with_context(ctx, node, indent, true)`

## Recent Fixes (Session Dec 29 - Continued pt7)

### Record Declaration Line-Wrapped Fixes
- **LineWrappedRecordDeclaration.java**: Fixed all 6 missing violations
- Added handling for `formal_parameters` (record's parentheses)
- Added handling for `super_interfaces` (implements clause)
- Added `check_super_interfaces_type_list` for type names on continuation lines

**Key changes to `check_class_declaration`:**
- Check closing `)` of record formal_parameters on continuation lines
- Check opening `(` for nested records
- Check `implements` keyword and type_list for implements clauses

## Recent Fixes (Session Dec 29 - Continued pt6)

### Lambda Expression Block Indent Fixes
- **Lambda3.java**: Fixed 2 missing violations for misaligned lambda block content
- **Lambda6.java**: Fixed 14 extra violations (false positives with lineWrappingIndentation=0)
- **Lambda8.java**: Fixed 1 extra violation (closing brace at line-wrapped position)
- **Lambda1.java**: Maintained correct behavior for nested lambdas in method calls

**Key changes to `check_lambda_expression`:**
- When lambda NOT at start of line but line is over-indented → parent statement is misaligned, use expected
- When lambda at start of line at `indent + basicOffset` or `indent + lineWrap` → use combined indent
- Otherwise use lambda's actual position as base

## Recent Fixes (Session Dec 29 - Continued pt5)

### While Statement and Binary Expression Fixes
- **InputIndentationInvalidWhileIndent.java**: Fixed all 3 remaining missing violations
- Fixed deeply nested binary expression threshold logic

**Changes made:**
- Updated `check_while_statement` to use actual position for misaligned statements
- Changed deep nesting threshold from absolute to relative (`indent + 2*lineWrap`)
- Updated expected_indent heuristic: if `expr_start < indent + lineWrap`, use `indent` (nested context), else use `indent + lineWrap` (statement context)
- For binary expressions in method call arguments, pass `indent` instead of `nested_indent` to avoid double-counting lineWrap

### If Statement Condition Fixes
- **InputIndentationInvalidIfIndent2.java**: Fixed all 5 missing violations for binary expression continuations in if-conditions
- **InputIndentationValidIfIndent.java**: Fixed lparen/rparen checking with correct line-wrapped indent
- **InputIndentationAndroidStyle.java**: Fixed misaligned if-statement expression checking

**Changes made to `check_if_statement`:**
- Use line-wrapped indent for lparen on continuation lines
- Check condition content with line-wrapped indent
- Accept both indent and line-wrapped indent for rparen (handles `) {` vs `)` alone)
- For misaligned if statements, use actual position for expression continuation calculation

**Changes made to `check_binary_expression`:**
- Use `ctx.column_from_node(node)` instead of `ctx.get_line_start(expr_line)` to get actual expression start column
- Determine expected_indent based on expression position:
  - If `indent > expr_start`: expression is under-indented, use context indent
  - Otherwise: use `indent + lineWrappingIndentation`

## Previous Fixes (Session Dec 29 - Continued pt4)

### While/Do-While Condition Fixes
- **InputIndentationInvalidWhileIndent.java**: Fixed all 6 missing violations (was Missing: 6)
- **InputIndentationInvalidDoWhileIndent.java**: Fixed all 5 missing violations (was Missing: 5)

**Changes made:**
- Added condition checking to `check_while_statement` and `check_do_while_statement`
- Check binary expressions inside conditions via `check_expression`
- Check opening paren if on its own line (should be at statement indent)
- Check closing paren if on its own line (should be at statement indent)
- Check condition content (identifiers, expressions) if on own line

### Binary Expression Lenient Mode Fixes
- **Fixed all 6 extra violations** (false positives):
  - InputIndentationIfAndParameter.java: 2 extra → 0 extra
  - InputIndentationNewChildrenSevntuConfig.java: 1 extra → 0 extra
  - InputIndentationValidAssignIndent.java: 1 extra → 0 extra
  - InputIndentationCheckMethodParenOnNewLine1.java: 2 extra → 0 extra

**Root cause and fix:**
1. For deeply nested binary expressions (inside method call arguments), `indent` accumulates with each nesting level, making `base_line_wrapped` too high
2. For expressions where the start is misaligned, continuations shouldn't be based on the wrong start position
3. Special case: continuations at exactly `expr_start` should be accepted (aligned with expression)

**Changes made to `check_binary_expression`:**
- Compute `expected_indent = min(expr_based, indent)` when expression is misaligned from context
- Accept continuations exactly at `expr_start` as valid alignment
- For deeply nested cases (expr_start > 3*lineWrap), use `indent` as floor instead of `indent + lineWrap`

## Previous Fixes (Session Dec 29 - Continued pt3)

### Anonymous Class Brace Fixes
- **AnonymousClassInMethodCurlyOnNewLine.java**: Fixed all 6 missing violations

### Local Class and Type Continuation Fixes
- **InvalidClassDefIndent1.java**: Fixed all 9 missing violations

### Binary Expression and Text Block Fixes
- **MultilineStatements.java**: Fixed all 4 missing violations

**Pattern to look for in test files:** `exp:>=N` means lenient mode (accept N or higher).

---

## Known Issues

### Extra Violations on Test Fixtures: RESOLVED ✓
All extra violations on checkstyle test fixtures have been fixed. (0 extra)

### Real-World Code - IMPROVED

All three projects pass checkstyle with 0 indentation violations. After Dec 30 fix:

| Project | Checkstyle | Lintal (Before) | Lintal (After) | Reduction |
|---------|------------|-----------------|----------------|-----------|
| artio   | 0          | 1813            | 275            | 85%       |
| agrona  | 0          | 79              | 18             | 77%       |
| aeron   | 0          | 160             | 161            | -         |

**Remaining false positives (to investigate):**
1. **`expr` child violations** - expression continuation in some contexts
2. **Annotation array initializers** - `@SuppressWarnings({"foo", "bar"})` pattern
3. **Aeron-specific patterns** - different from artio/agrona style

### Remaining Missing Violations (60 total)

| Category | Files | Missing |
|----------|-------|---------|
| Record declarations | RecordsAndCompactCtors | 2 |
| Array init | InvalidArrayInit files | ~12 |
| Lambda expressions | Lambda (arrow edge cases) | 2 |
| Switch statements | InvalidSwitchIndent, SwitchExpressionWrapping | 6 |
| Method calls | MethodCallLineWrap, ChainedMethodCalls | 4 |
| Other | Various | ~46 |

---

## Next Steps

### Priority 1: Remaining Real-World False Positives (~450 total)
1. **`expr` child violations** - Expression continuation patterns (most common remaining issue)
2. **Annotation array initializers** - `@SuppressWarnings({"foo", "bar"})` pattern
3. **Aeron-specific patterns** - Different code style from artio/agrona

### Priority 2: Remaining Test Fixture Issues (60 missing)
1. **Record declarations** - Add record-specific handlers
2. **Switch statements** - Fix switch expression wrapping
3. **Method call continuations** - Handle chained method calls
4. **Array init edge cases** - Remaining array initialization patterns

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
