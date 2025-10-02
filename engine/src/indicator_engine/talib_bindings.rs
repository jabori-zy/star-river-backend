#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
#[cfg(target_os = "windows")]
include!("bindings_windows.rs");

#[cfg(target_os = "macos")]
include!("bindings_macos1.rs");