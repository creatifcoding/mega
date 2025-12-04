#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
// lazy_cell is stable since Rust 1.80, no feature flag needed
#![allow(unused_imports)]

#![cfg_attr(feature = "strict", deny(warnings))]

extern crate libc;
extern crate lisp_util;
extern crate regex;

mod docfile;

pub use crate::{
    // Used by make-docfile
    docfile::scan_rust_file,
};
