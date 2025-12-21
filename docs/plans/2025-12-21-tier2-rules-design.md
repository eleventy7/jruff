# Tier 2 Rules Implementation Design

Implementing 6 remaining Tier 2 checkstyle rules with auto-fix support.

## Rules Overview

| Rule | Category | Complexity | Priority |
|------|----------|------------|----------|
| OneStatementPerLine | coding | Low | 1 |
| MultipleVariableDeclarations | coding | Low | 2 |
| OperatorWrap | whitespace | Medium | 3 |
| EmptyLineSeparator | whitespace | High | 4 |
| SimplifyBooleanReturn | coding | Medium | 5 |
| DeclarationOrder | coding | High | 6 |

## Directory Structure

```
crates/lintal_linter/src/rules/
├── coding/                          # NEW directory
│   ├── mod.rs
│   ├── one_statement_per_line.rs
│   ├── multiple_variable_declarations.rs
│   ├── simplify_boolean_return.rs
│   └── declaration_order.rs
└── whitespace/
    ├── operator_wrap.rs             # NEW
    └── empty_line_separator.rs      # NEW
```

## Rule 1: OneStatementPerLine

**Checkstyle module:** `com.puppycrawl.tools.checkstyle.checks.coding.OneStatementPerLineCheck`

**Violations:**
- Multiple statements on same line: `int a; int b;`
- Statement spanning lines with semicolon on different line: `one = 1\n; two = 2;`

**Exceptions (NOT violations):**
- For-loop headers: `for (int i = 0; i < 10; i++)`
- Try-with-resources (when `treatTryResourcesAsStatement=false`)
- Resources in try-with-resources separated by `;` on same line (configurable)

**Configuration:**
```xml
<module name="OneStatementPerLine">
    <property name="treatTryResourcesAsStatement" value="false"/>
</module>
```

**Implementation approach:**
1. Walk all statement nodes
2. Track line numbers of statement starts
3. Flag when two statements start on same line (excluding for-loop init)
4. For try-with-resources, optionally treat each resource as statement

**Fix strategy:** Insert `\n` + indentation before second statement.

**Message:** `"Only one statement per line allowed."`

## Rule 2: MultipleVariableDeclarations

**Checkstyle module:** `com.puppycrawl.tools.checkstyle.checks.coding.MultipleVariableDeclarationsCheck`

**Two violation types:**

1. **Comma-separated variables in single declaration:**
   ```java
   int i, j;  // violation: "Each variable declaration must be in its own statement."
   ```

2. **Multiple declarations on same line:**
   ```java
   int i; int j;  // violation: "Only one variable definition per line allowed."
   ```

**Exceptions:**
- For-loop initializers: `for (int i = 0, j = 0; ...)` - allowed

**Configuration:** None

**Implementation approach:**
1. For local_variable_declaration nodes, check if multiple declarators exist
2. Track declaration start lines to detect same-line declarations
3. Skip for-loop initializers

**Fix strategy:**
- Comma-separated: Split into separate statements with proper indentation
- Same-line: Insert newline between declarations

**Messages:**
- `"Each variable declaration must be in its own statement."`
- `"Only one variable definition per line allowed."`

## Rule 3: OperatorWrap

**Checkstyle module:** `com.puppycrawl.tools.checkstyle.checks.whitespace.OperatorWrapCheck`

**What it detects:** Operator position when expression spans multiple lines.

**Options:**
- `nl` (default): Operator must be on new line
- `eol`: Operator must be at end of line

```java
// option=nl violation:
int x = 1 +     // '+' should be on new line
    2;

// option=nl correct:
int x = 1
    + 2;
```

**Tokens checked:** `+`, `-`, `*`, `/`, `%`, `?`, `:`, `&&`, `||`, `&`, `|`, `^`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `<<`, `>>`, `>>>`, `instanceof`, `&` (type bounds)

**Configuration:**
```xml
<module name="OperatorWrap">
    <property name="option" value="nl"/>
    <property name="tokens" value="PLUS, MINUS, STAR, ..."/>
</module>
```

**Fix strategy:** Move operator to correct line, preserving indentation.

**Messages:**
- `"'+' should be on a new line."` (for nl option)
- `"'+' should be on the previous line."` (for eol option)

## Rule 4: EmptyLineSeparator

**Checkstyle module:** `com.puppycrawl.tools.checkstyle.checks.whitespace.EmptyLineSeparatorCheck`

**What it detects:** Missing or excessive blank lines between:
- Package declaration and imports
- Import groups
- Class members (fields, methods, constructors, inner classes)

**Configuration options:**
- `allowNoEmptyLineBetweenFields` (default: false)
- `allowMultipleEmptyLines` (default: true)
- `allowMultipleEmptyLinesInsideClassMembers` (default: true)
- `tokens` - which elements to check

**Complexity:** High due to many configuration options and context-dependent behavior.

**Fix strategy:** Insert or remove blank lines as needed.

## Rule 5: SimplifyBooleanReturn

**Checkstyle module:** `com.puppycrawl.tools.checkstyle.checks.coding.SimplifyBooleanReturnCheck`

**What it detects:**
```java
// Violation - can simplify to "return !even"
if (even) {
    return false;
} else {
    return true;
}

// Violation - can simplify to "return !even"
if (!even) {
    return true;
} else {
    return false;
}
```

**Pattern matching:**
- `if (cond) { return true; } else { return false; }` → `return cond;`
- `if (cond) { return false; } else { return true; }` → `return !cond;`

**Configuration:** None

**Fix strategy:** Replace entire if-else with simplified return statement.

**Message:** `"Conditional logic can be removed."`

## Rule 6: DeclarationOrder

**Checkstyle module:** `com.puppycrawl.tools.checkstyle.checks.coding.DeclarationOrderCheck`

**Expected order in class:**
1. Static variables (public → protected → package → private)
2. Instance variables (public → protected → package → private)
3. Constructors
4. Methods

**Configuration:**
- `ignoreConstructors` (default: false)
- `ignoreModifiers` (default: false)

**Complexity:** Very high - requires reordering potentially large code blocks.

**Fix strategy:** Reorder class members. This produces large diffs and may be marked as "unsafe" fix.

## Testing Strategy

Each rule will have:
1. Unit tests in the rule file
2. Checkstyle compatibility tests using fixtures from `target/checkstyle-tests/`

Test file pattern:
```
crates/lintal_linter/tests/checkstyle_onestatementperline.rs
crates/lintal_linter/tests/checkstyle_multiplevariabledeclarations.rs
...
```

## Implementation Order

1. **Phase A: Setup coding/ module**
   - Create `crates/lintal_linter/src/rules/coding/mod.rs`
   - Export from main rules module

2. **Phase B: OneStatementPerLine**
   - Implement rule with treatTryResourcesAsStatement config
   - Add checkstyle compatibility tests
   - Register in registry

3. **Phase C: MultipleVariableDeclarations**
   - Implement both violation types
   - Handle for-loop exception
   - Add checkstyle compatibility tests

4. **Phase D: OperatorWrap**
   - Add to whitespace/ directory
   - Implement nl/eol options
   - Add checkstyle compatibility tests

5. **Phase E: EmptyLineSeparator**
   - Complex rule with many config options
   - Start with basic member separation
   - Add config options incrementally

6. **Phase F: SimplifyBooleanReturn**
   - Pattern matching for if-else returns
   - Semantic fix generation

7. **Phase G: DeclarationOrder**
   - Detection of out-of-order members
   - Optional: unsafe fix for reordering
