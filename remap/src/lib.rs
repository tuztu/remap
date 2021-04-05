#![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]

#[macro_use]
extern crate anyhow;
#[cfg_attr(test, macro_use)]
extern crate remap_derive;

pub mod mysql;
pub mod config;
pub mod template;
pub mod extend;
pub mod arguments;
