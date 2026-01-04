# Phase 3 Implementation Plan: Advanced Mixins & Namespaces

This phase focuses on enabling advanced Less features: Recursion (Loops), Rulesets as Mixins, and Namespaces.

## 1. Recursion Safety & Loops
**Goal**: Support recursive mixin calls (e.g., for loops) while preventing stack overflows.

-   **Compiler Update**:
    -   Add `recursion_depth: usize` and `max_recursion_depth: usize` (default 100) to `Compiler` struct.
    -   Increment depth in `compile_mixin_call` before pushing scope, decrement after popping.
    -   Throw `Error::RecursionLimitExceeded` if depth limit is reached.
-   **Testing**:
    -   Create `tests/test_loops.rs`.
    -   Test standard recursive loop pattern (with guards).
    -   Test infinite recursion (ensure it fails gracefully, not crash).

## 2. Rulesets as Mixins (Implicit Mixins)
**Goal**: Allow standard CSS rulesets (e.g., `.class { ... }`) to be called as mixins. This is a prerequisite for Namespaces.

-   **Compiler Update**:
    -   Modify `compile_rule` in `src/compiler.rs`.
    -   Before/During compilation of a Rule, register it as a `MixinDefinition` in the current scope.
        -   **Name**: The selector string (e.g., `.class`).
        -   **Body**: The rule's nested statements (declarations, nested rules).
        -   **Parameters**: None (Implicit mixins take no arguments, though Less allows `()` to be omitted).
    -   Ensure this registration happens in a way that allows `lookup_mixin` to find it.

## 3. Namespaces
**Goal**: Support `#namespace > .mixin()` syntax.

-   **AST Update**:
    -   Update `MixinCall` struct in `src/ast/mod.rs`:
        -   Change `name: String` to `selector: Selector` (or keep `String` but parse it as a path).
        -   *Decision*: Use `name: String` but enhance `parse_mixin_call` to handle `>` and spaces, storing the full path string (e.g., `#ns > .mixin`). The Compiler will then parse this path.
-   **Compiler Update**:
    -   Implement `resolve_mixin_path(&str)` in `Compiler`.
    -   Logic:
        1.  Split path by `>` or descendant combinators.
        2.  Lookup the first segment (e.g., `#ns`) in the current scope.
        3.  If found (as a Mixin or Rule), "enter" it.
            -   This requires a way to search *inside* a `MixinDefinition`.
            -   Since `MixinDefinition` has a `body` (Vec<Statement>), we need a helper to scan the body for the next segment (e.g., `.mixin`).
    -   Update `compile_mixin_call` to use this resolution logic.
-   **Testing**:
    -   Create `tests/test_namespaces.rs`.
    -   Test calling mixins inside rulesets (`#ns > .mixin()`).
    -   Test nested namespaces (`#a > #b > .c()`).

## Execution Steps
1.  **Step 1: Recursion Safety**: Implement depth tracking and limit. Add loop tests.
2.  **Step 2: Implicit Mixins**: Register Rules as Mixins. Add tests for calling classes.
3.  **Step 3: Namespaces**: Implement path resolution logic and namespace support. Add namespace tests.
