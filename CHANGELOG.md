v0.3.9 - 2023-02-07:
  * Bump minimum Rust version to 1.66.
  * Remove use of `proc_macro_span` feature now that `proc_macro::Span::source_text` has been stabilized.

v0.3.8 - 2023-01-22:
  * Reduce risk of interleaved output of concurrent tests when running tests with `--no-capture`.
  * Fix minimum required `proc-macro2` version.

v0.3.7 - 2022-11-21:
  * Bump required Rust version to 1.65 for `let ... else { }`.
  * Fix ambiguous patterns without captures in `let_assert!()`.

v0.3.6:
  * Update dependencies.

v0.3.5:
  * Fix Windows compatibility by using `atty` crate for TTY detection.

v0.3.4:
  * Rename internal doc-hidden items to avoid issues with `use assert2::*`.

v0.3.3:
  * Fix stringification of non-sized types.

v0.3.2:
  * Support mutable captures in `let_assert!(...)`.
  * Support capturing by reference in `let_assert!(...)` (as long as Rust allows it).

v0.3.1:
  * Use `$crate` to avoid the need for a direct dependency on `assert2`.

v0.3.0:
  * Use stabilized `proc_macro` expressions in place of `proc_macro_hack`.
  * Improve display of macro fragments on nightly.
  * Format expressions nicer on `stable` and `beta`.
  * Work around hygiene bug in Rust compiler ([issue #67062]).

[issue #67062]: https://github.com/rust-lang/rust/issues/67062

v0.2.1:
  * Prevent `assert!(a == b)` from consuming `a` or `b`.

v0.2.0:
  * Add feature-gated "let-assert" macro for nightly.
  * Implement semi-standard CLICOLOR / CLICOLOR_FORCE standard correctly.

v0.1.2:
  * Add `debug_assert!(...)` for parity with the standard library.

v0.1.1:
  * Synchronize README with library documentation.

v0.1.0:
  * Fully compatible with Rust stable.
  * Only use `proc_macro_span` on nightly.
  * Tweak colors in output.

v0.0.9:
  * Use `proc-macro-hack` to avoid `feature(proc_macro_hygiene)`.
  * Use auto-deref specialization to avoid `feature(proc_macro_hygiene)`.

v0.0.8:
  * Fix compilation error in assert!()
  * Limit scope of generated temporary variables

v0.0.7:
  * Update documentation.

v0.0.6:
  * Support additional arguments to print custom messages with failures.

v0.0.5:
  * Update documentation.
  * Fix images in documentation.

v0.0.4:
  * Support pattern matching with `let` expressions in assertions.
