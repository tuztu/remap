#![allow(unused_imports, deprecated, unused_must_use, unused_mut, unused_variables, dead_code, unreachable_code)]

#[macro_use]
extern crate anyhow;
#[cfg_attr(test, macro_use)]
extern crate remap_derive;

pub use arguments::Args;
pub use extend::Remap;
pub use template::mysql::MySqlTemplate;

pub(crate) mod template;
pub(crate) mod extend;
pub(crate) mod arguments;
