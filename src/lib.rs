#![recursion_limit = "512"]
#[macro_use]
mod macros;

#[cfg(feature = "preserve_order")]
extern crate regex;

/// Edn type implementation
pub mod edn;

/// Serialization module for most possible types.
/// Tuples are limited between `(A, B)` and `(A, B, C, D, E, F)`, any other tuple needs to be implemented by the `trait Serialize`.
/// This module requires `#[macro_use]` for `structs`.
///
/// Example:
/// ```rust
/// #[macro_use] extern crate edn_rs;
///
/// use std::collections::{BTreeMap, BTreeSet};
/// use crate::edn_rs::Serialize;
///
/// fn main() {
///     ser_struct!{
///         #[derive(Debug)]
///         struct Edn {
///             map: BTreeMap<String, Vec<String>>,
///             set: BTreeSet<i64>,
///             tuples: (i32, bool, char),
///         }
///     };
///     let edn = Edn {
///         map: map!{"this is a key".to_string() => vec!["with".to_string(), "many".to_string(), "keys".to_string()]},
///         set: set!{3i64, 4i64, 5i64},
///         tuples: (3i32, true, 'd')
///     };
///     println!("{}",edn.serialize());
///     // { :map {:this-is-a-key ["with", "many", "keys"]}, :set #{3, 4, 5}, :tuples (3, true, \d), }
/// }
///```
pub mod serialize;

use edn::utils::{replace_char, replace_keywords};
mod deserialize;
/// `json_to_edn` receives a json string and parses its common key-values to a regular EDN format.
/// tested examples are:
/// 1. `"{\"hello world\": \"julia\"}"` becomes `"{:hello-world \"julia\"}"`
/// 2. `"{\"hello\": null}"` becomes `"{:hello nil}"`
/// 3. `{\"hello\": 'c'}` becomes `"{:hello \\c}"`
/// 4. `"{\"multi_string with underscore\": 545643}"` becomes `"{:multi-string-with-underscore 545643}"`
///
/// ```
/// use edn_rs::json_to_edn;
///
/// fn emits_helloworld_edn() {
///     let json = String::from("{\"hello\": \"world\"}");
///     let edn = String::from("{:hello \"world\"}");
///
///     assert_eq!(edn, json_to_edn(json));
/// }
///
/// fn emits_vec_of_map_edn() {
///     let json = String::from("[{\"hello\": \"world\"}, {\"hello\": \"julia\"}, {\"hello\": \"serde\"}");
///     let edn = String::from("[{:hello \"world\"} {:hello \"julia\"} {:hello \"serde\"}]");
///
///     assert_eq!(edn, json_to_edn(json));
/// }
/// ```
pub fn json_to_edn(json: String) -> String {
    let edn_aux = replace_keywords(json);
    let edn = replace_char(edn_aux);
    edn.replace("null", "nil")
}

pub use deserialize::from_str;
pub use edn::Error as EdnError;
pub use edn::{Double, Edn, List, Map, Set, Vector};
pub use serialize::Serialize;
