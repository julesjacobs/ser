//! Code that uses the ISL library.

#![allow(unsafe_op_in_unsafe_fn)]


/// Based on https://rust-lang.github.io/rust-bindgen/tutorial-4.html
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/isl_bindings.rs"));
}