# Repository Guidelines for egui-keyframe

## Project Structure

Core library code lives in `src/`, with `lib.rs` exposing the public API. The `src/core/` module contains fundamental types (`TimeTick`, `Keyframe`, `Track`). The `src/widgets/` module provides egui widgets (`CurveEditor`, `TimeRuler`, `BoundingBox`). The `src/spaces.rs` module handles coordinate transformations between time/value and screen space.

## Build, Test, and Development Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run tests with all features
cargo test --all-features

# Format code
cargo fmt --all

# Run clippy linter
cargo clippy --all-targets --all-features -- -W warnings
```

## Coding Style & Naming Conventions

- Follow standard Rust style: four-space indentation, `snake_case` for modules/functions, and `CamelCase` for types.
- Write idiomatic Rust. Avoid patterns from imperative languages that can be expressed more elegantly.
- PREFER functional style over imperative style. Use `map`/`filter`/`collect` instead of for loops with `push`.
- PREFER direct initialization of collections.
- AVOID unnecessary allocations, conversions, copies.
- AVOID using `unsafe` code unless absolutely necessary.
- AVOID return statements; structure functions with if-else blocks instead.
- Keep public APIs documented with `///` comments.
- Run `cargo fmt --all` before committing.
- Lint with `cargo clippy --all-targets --all-features` and address warnings.

### Naming Conventions (Rust API Guidelines)

- **Casing**: `UpperCamelCase` for types/traits/variants; `snake_case` for functions/methods/modules/variables; `SCREAMING_SNAKE_CASE` for constants/statics.
- **Conversions**: `as_` for cheap borrowed-to-borrowed; `to_` for expensive conversions; `into_` for ownership-consuming conversions.
- **Getters**: No `get_` prefix (use `width()` not `get_width()`).
- **Iterators**: `iter()` for `&T`, `iter_mut()` for `&mut T`, `into_iter()` for `T` by value.
- **Tests**: NEVER use `test_` prefix/suffix in test function names.

## Testing Guidelines

- Tests rely on Rust's built-in harness.
- **Test Naming Convention**: Test functions should NOT be prefixed with `test_`. The `#[test]` attribute already indicates it's a test.
- **CRITICAL: ALWAYS run `cargo test` and ensure the code compiles and tests pass WITHOUT ANY WARNINGS BEFORE committing!**
- **CRITICAL: Address ALL warnings before EVERY commit!** This includes unused imports, dead code, deprecated API usage.
- ALWAYS run `cargo clippy --fix` before committing.

## Documentation Guidelines

- All code comments MUST end with a period.
- All doc comments should also end with a period unless they're headlines.
- All comments must be on their own line. Never put comments at the end of a line of code.
- All references to types, keywords, symbols etc. MUST be enclosed in backticks.

## Writing Instructions

These instructions apply to any communication as well as any documentation you write:

- Be concise.
- Use simple sentences. Feel free to use technical jargon.
- Do NOT overexplain basic concepts. Assume the user is technically proficient.
- AVOID flattering, corporate-ish or marketing language.
- AVOID vague and/or generic claims.

## Error Handling

- Prefer `?` operator and `Result` return types over `.unwrap()`.
- Use `.ok_or()` or `.ok_or_else()` to convert `Option` to `Result` with meaningful errors.
- `.unwrap()` is permitted ONLY when an invariant guarantees the value will never be `None`/`Err`, with a `// SAFETY:` comment explaining why.

## Traits

- All public-facing types should implement `Debug`, `Clone`, `PartialEq` and `Copy` if directly derivable.
