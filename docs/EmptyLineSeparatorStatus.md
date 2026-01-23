# EmptyLineSeparator Rule Status

## Current Detection Rate: 100.0%

| Metric | Value |
|--------|-------|
| Expected violations | 155 |
| Found violations | 155 |
| Correct matches | 155 |
| Missing (false negatives) | 0 |
| False positives | 0 |
| Detection rate | 100.0% |

## Running the Tests

### Run all EmptyLineSeparator compatibility tests

```bash
cargo test --package lintal_linter --test checkstyle_emptylineseparator
```

### Run with detailed output to see detection rate

```bash
cargo test --package lintal_linter --test checkstyle_emptylineseparator test_all_fixtures -- --nocapture
```

### Run a specific fixture test with details

```bash
cargo test --package lintal_linter --test checkstyle_emptylineseparator test_with_emoji_detailed -- --nocapture
```

## Test Fixtures

The tests use checkstyle's own test fixtures, cloned to `target/checkstyle-tests/`. There are 49 fixture files covering various scenarios:

- Basic separation rules
- Multiple empty lines detection
- Empty lines inside class members
- Comments and javadoc handling
- Package/import separation
- Records and compact constructors
- Interface fields
- Enum members

## Configuration Options Tested

| Option | Default | Description |
|--------|---------|-------------|
| `allowNoEmptyLineBetweenFields` | `false` | Allow fields without blank lines between them |
| `allowMultipleEmptyLines` | `true` | Allow more than one consecutive empty line |
| `allowMultipleEmptyLinesInsideClassMembers` | `true` | Allow multiple empty lines inside method/constructor bodies |

## Violation Types

1. **ShouldBeSeparated**: Element should have a blank line before it
2. **TooManyEmptyLines**: Element has more than 1 empty line before it
3. **TooManyEmptyLinesAfter**: Closing brace has more than 1 empty line after last content
4. **TooManyEmptyLinesInside**: More than 1 consecutive empty line inside a method/constructor
5. **CommentTooManyEmptyLines**: Comment has more than 1 empty line before it

## Known Limitations

### Missing Violations

None known.

### False Positives

None known.

## Implementation Notes

### Key Logic

1. **Comment attached to code**: When a comment immediately follows a statement (not a brace), violations are reported on the code line, not the comment. This matches checkstyle behavior.

2. **Array initializer handling**: For gaps with no intervening comments, violations are reported on both sides of the gap (element before AND element after).

3. **Brace detection**: Lines containing only braces are tracked separately. Comments following braces are treated as standalone, not attached to the brace.

4. **Nested block skipping**: Nested class bodies, interface bodies, and array initializers are checked separately to avoid duplicate reporting.

### Source Files

- Rule implementation: `crates/lintal_linter/src/rules/whitespace/empty_line_separator.rs`
- Test file: `crates/lintal_linter/tests/checkstyle_emptylineseparator.rs`

## Debugging Tips

### View AST structure for a Java file

```bash
echo 'class Test { void foo() { } }' | ./target/debug/dump_java_ast
```

### Create a debug script

Create a file like `/tmp/debug_test.rs` with custom test logic, then compile and run against the lintal libraries.

### Check specific fixture violations

```bash
# Run detailed test for a specific fixture
cargo test --package lintal_linter --test checkstyle_emptylineseparator test_postfix_corner_cases_detailed -- --nocapture
```

## History

- Started at ~97.4% detection rate
- Fixed line number reporting for TooManyEmptyLinesInside
- Added smart "comment attached to code" detection
- Added array initializer gap handling for both sides
- Fixed test parser for "violation above this line" pattern
- Current: 100.0% detection rate
