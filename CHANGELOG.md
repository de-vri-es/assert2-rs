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
