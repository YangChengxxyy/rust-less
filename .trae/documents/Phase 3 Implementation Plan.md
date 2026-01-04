# Phase 3: Advanced Extend Functionality Implementation Plan

## 1. Goal Setting
**Primary Objective**: Enhance the current `:extend` implementation to support the `all` keyword (partial matching) and selector-attached `:extend` syntax (e.g., `.a:extend(.b) {}`).

**Key Performance Indicators (KPIs)**:
-   **Functionality**:
    -   Pass `test_parse_extend_all` with partial matching logic verified in compilation output.
    -   Support `.selector:extend(...)` syntax in rules (not just `&:extend` statements).
-   **Quality**: No regression in existing 95+ unit tests.
-   **Performance**: Extend matching overhead remains negligible for stylesheets < 1000 lines.

**Timeline**:
-   **Milestone 1**: `all` keyword support (Estimated: 2 steps)
-   **Milestone 2**: Selector-attached extend syntax (Estimated: 2 steps)
-   **Milestone 3**: Integration and Verification (Estimated: 1 step)

## 2. Resource Assessment
-   **Resources**:
    -   **AI Assistant**: Code implementation, refactoring, and test generation.
    -   **User**: Review, direction, and confirmation.
-   **Gaps**:
    -   Current `ExtendRegistry` uses `HashMap` for exact matches. Need to add iteration/search capability for `all` keyword.
    -   `Parser` currently parses `&:extend` as a `Statement`. Need to identify `:extend` pseudo-classes within `Selector` parsing logic.

## 3. Risk Assessment
| Risk | Probability | Impact | Mitigation Strategy |
| :--- | :--- | :--- | :--- |
| **Performance Degradation** | Medium | Low (for now) | Use efficient string matching; optimize `ExtendRegistry` lookup. Keep `all` checks separate from exact checks. |
| **Parser Complexity** | Medium | High | Re-use `parse_extend` logic or shared helper functions. Ensure `:extend` pseudo-class is correctly distinguished from normal pseudo-classes. |
| **Infinite Recursion** | Low | High | Maintain the "Extend is not recursive" principle of Less. Ensure extenders don't trigger further extends in the same pass. |

## 4. Task Breakdown

### Milestone 1: `all` Keyword Support
-   **Task 3.1.1**: Refactor `ExtendRegistry` in `src/extend.rs`.
    -   Add `find_partial_matches(selector: &str) -> Vec<String>`.
    -   Logic: Iterate all registered extends where `all` is true. Check if `target` is contained in `selector`.
    -   Owner: AI.
-   **Task 3.1.2**: Update `Compiler::compile_rule` in `src/compiler.rs`.
    -   Call `find_partial_matches` in addition to `find_exact_matches`.
    -   Perform string replacement: `selector.replace(target, extender)`.
    -   Owner: AI.

### Milestone 2: Selector-Attached Syntax
-   **Task 3.2.1**: Update `ExtendCollector` in `src/extend.rs`.
    -   Implement visiting of `Selector` nodes.
    -   Look for `SimpleSelector::PseudoClass` with name "extend".
    -   Extract arguments, resolve selectors, and register in `ExtendRegistry`.
    -   Owner: AI.
-   **Task 3.2.2**: Update `Compiler` to strip `:extend` pseudo-classes.
    -   When compiling selectors to string, filter out `:extend` pseudo-classes so they don't appear in CSS output.
    -   Owner: AI.

### Milestone 3: Verification
-   **Task 3.3.1**: Add comprehensive tests in `tests/test_extend_advanced.rs`.
    -   Test `all` keyword with partial matches (`.c .a:extend(.a all)`).
    -   Test selector-attached syntax (`.a:extend(.b) {}`).
    -   Test multiple extends in one selector.

## 5. Progress Monitoring
-   **Mechanism**: Step-by-step tool execution and test verification.
-   **Checkpoints**:
    -   After Task 3.1.2: Run `test_extend_compilation` and new `all` tests.
    -   After Task 3.2.2: Run new selector syntax tests.

## 6. Quality Assurance
-   **Standards**: Code must compile without warnings. All new logic must be covered by tests.
-   **Acceptance**:
    -   Existing tests pass.
    -   New tests for `all` and selector-attached syntax pass.
    -   No "unimplemented" panics for supported syntax.
