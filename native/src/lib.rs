//! Native C ABI final link of Khronos Vulkan-Loader against Scarlet's Rust CRT.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
extern crate scarlet_c;
core::arch::global_asm!(".hidden __scarlet_getauxval", ".hidden __scarlet_start");
include!(concat!(env!("OUT_DIR"), "/loader_exports.rs"));
