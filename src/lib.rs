#![feature(proc_macro_hygiene)]
#![feature(specialization)]

mod maybe_debug;

#[doc(hidden)]
pub mod print;

pub use check_macros::check;
