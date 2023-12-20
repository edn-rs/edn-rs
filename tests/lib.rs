#![recursion_limit = "512"]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod deserialize;
pub mod deserialize_sets;
pub mod emit;
pub mod parse;
pub mod parse_sets;
pub mod ser;
