# Check Path Dispatch Table Design

## Goal
Reduce `check`-path CPU time by avoiding calling every rule on every CST node. Keep the existing traversal (`TreeWalker`) and rule API stable, but add an optional dispatch filter that only runs rules for relevant node kinds.

## Approach
Build a dispatch table that maps tree-sitter node kind IDs to the rules that care about them. During traversal, look up the current node kind ID and run only those rules, plus a small catch-all set for rules that do not declare kinds. This preserves correctness while shrinking the hot inner loop.

## API Changes
Extend the `Rule` trait with an optional method:

```
fn relevant_kinds(&self) -> &'static [&'static str] { &[] }
```

Empty means the rule is a catch-all and still runs for every node.

## Dispatch Table Construction
- Build once per run after `rules` are created.
- Use the Java tree-sitter `Language` to determine `node_kind_count()` and to map kind strings to kind IDs.
- Store:
  - `per_kind: Vec<Vec<usize>>` indexed by kind ID, each holding rule indices
  - `catch_all: Vec<usize>` for rules with empty `relevant_kinds`

Unknown kinds (strings not recognized by the language) are ignored, with a debug-only warning to avoid noisy output.

## Runtime Flow
- In `check_file`, for each node from `TreeWalker`, get `kind_id` and iterate `dispatch.per_kind[kind_id]` plus `catch_all`.
- Keep suppression logic unchanged; it now runs only for rules relevant to that node.

## Rollout Plan
1. Add the `relevant_kinds` method with default `&[]` and integrate dispatch table with `catch_all` so behavior is unchanged.
2. Incrementally add `relevant_kinds` to high-traffic rules (Whitespace*, Indentation, Imports) and measure gains.

## Testing and Validation
- Existing rule tests should continue to pass.
- Benchmark before and after using `scripts/benchmark.py` (via `mise run benchmark`).
- Optional: add a debug-only counter to compare “rule invocations per node” before and after for spot checks.

## Risks
- If a rule forgets to include a relevant node kind, it may miss diagnostics. Mitigation: start with catch-all defaults and only narrow rules with clear, exhaustive kind sets, and validate with benchmarks on real repos.
