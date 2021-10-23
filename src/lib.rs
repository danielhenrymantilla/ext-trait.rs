#![cfg_attr(feature = "better-docs",
    cfg_attr(all(), doc = include_str!("../README.md")),
)]
#![no_std]
#![forbid(unsafe_code)]

#[cfg(COMMENTED_OUT)] // <- Remove this when used!
/// The crate's prelude.
pub
mod prelude {
    // …
}

#[doc(inline)]
pub use ::ext_trait_proc_macros::*;

// To get fancier docs, for each exported procedural macro, put the docstring
// here, on the re-export, rather than on the proc-macro function definition.
// Indeed, this way, the internal doc links will Just Work™.
#[cfg(COMMENTED_OUT)] // <- Remove this when used!
/// Docstring for the proc-macro.
pub use ::ext_trait_proc_macros::some_macro_name;

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod __ {
    pub use ::core;
}

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
