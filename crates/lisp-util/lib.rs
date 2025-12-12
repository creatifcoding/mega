#![cfg_attr(feature = "strict", deny(warnings))]

#[macro_use]
mod attributes;

// Used by lisp-macros and lisp-doc
pub use self::attributes::parse_lisp_fn;
