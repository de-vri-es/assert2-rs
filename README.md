# assert2

`assert!(...)` and `check!(...)` macros inspired by Catch2.

This crate is currently a work in progress.
It relies on a nightly compiler with the `proc_macro_hygiene`, `proc_macro_span` and `specialization` features.

As a user of the crate, you also need to enable the `proc_macro_hygiene` feature.

## Example
```rust
#![feature(proc_macro_hygiene)]
use assert2::check;

let mut vec = Vec::new();
vec.push(12);

check!(vec.len() == 2);
check!(&vec == &vec![10]);
```

![Example output](https://github.com/de-vri-es/assert2-rs/blob/v0.0.3/example.png)
