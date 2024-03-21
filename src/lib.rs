#![no_std]
#![recursion_limit = "512"]

#[macro_use]
mod macros;

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::string::String;

/// Edn type implementation
pub mod edn;

/// Serialization module for most possible types.
/// Tuples are limited between `(A, B)` and `(A, B, C, D, E, F)`, any other tuple needs to be implemented by the `trait Serialize`.
/// This module requires `#[macro_use]` for `structs`.
///
/// Example:
/// ```rust
/// use std::collections::{BTreeMap, BTreeSet};
/// use edn_derive::Serialize;
/// use edn_rs::{set, map, edn::Edn};
///
/// #[derive(Serialize)]
/// struct ExampleEdn {
///     map: BTreeMap<String, Vec<String>>,
///     set: BTreeSet<i64>,
///     tuples: (i32, bool, char),
/// }
/// fn main() {
///     let edn = ExampleEdn {
///         map: map!{"this is a key".to_string() => vec!["with".to_string(), "many".to_string(), "keys".to_string()]},
///         set: set!{3i64, 4i64, 5i64},
///         tuples: (3i32, true, 'd')
///     };
///     println!("{}", edn_rs::to_string(&edn));
///     // { :map {:this-is-a-key ["with", "many", "keys"]}, :set #{3, 4, 5}, :tuples (3, true, \d), }
/// }
///```
#[allow(clippy::needless_doctest_main)]
pub mod serialize;

#[cfg(feature = "json")]
pub(crate) mod json;

#[cfg(feature = "json")]
use alloc::borrow::Cow;
#[cfg(feature = "json")]
use alloc::string::ToString;

mod deserialize;
/// `json_to_edn` receives a json string and parses its common key-values to a regular EDN format. It requires feature `json`
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
#[cfg(feature = "json")]
#[allow(clippy::missing_panics_doc)] // Our regex's don't rely on user-input
pub fn json_to_edn<'a>(json: impl AsRef<str>) -> Cow<'a, str> {
    use regex::{Captures, Regex};

    // Convert string keys to EDN keywords
    let re = Regex::new(r#""\w*(\s\w*)*":"#).unwrap();
    let json = re.replace_all(json.as_ref(), |caps: &Captures<'_>| {
        let mut rcap = caps[0].replace(['\"', ':'], "").replace(['_', ' '], "-");
        rcap.insert(0, ':');
        rcap.to_string()
    });

    // Convert chars
    let c_re = Regex::new(r"'.'").unwrap();
    let json = c_re.replace_all(&json[..], |caps: &Captures<'_>| {
        let mut rcap = caps[0].replace('\'', "");
        rcap.insert(0, '\\');
        rcap.to_string()
    });

    json.replace("null", "nil").into()
}

pub use deserialize::{from_edn, from_str, Deserialize};
pub use edn::error::Error as EdnError;
#[cfg(feature = "sets")]
pub use edn::Set;
pub use edn::{Edn, List, Map, Vector};
pub use serialize::Serialize;

/// Function for converting Rust types into EDN Strings.
/// For it to work, the type must implement the Serialize trait.
/// Use `#[derive(Serialize)]` from `edn-derive` crate.
///
/// Example:
/// ```rust
/// use std::collections::{BTreeMap, BTreeSet};
/// use edn_derive::Serialize;
/// use edn_rs::{set, map, edn::Edn};
///
/// #[derive(Debug, Serialize)]
/// struct ExampleEdn {
///     map: BTreeMap<String, Vec<String>>,
///     set: BTreeSet<i64>,
///     tuples: (i32, bool, char),
/// }
///
/// fn main() {
///     let edn = ExampleEdn {
///         map: map!{"this is a key".to_string() => vec!["with".to_string(), "many".to_string(), "keys".to_string()]},
///         set: set!{3i64, 4i64, 5i64},
///         tuples: (3i32, true, 'd')
///     };
///     println!("{}", edn_rs::to_string(&edn));
///     // { :map {:this-is-a-key ["with", "many", "keys"]}, :set #{3, 4, 5}, :tuples (3, true, \d), }
/// }
///```
#[allow(clippy::needless_doctest_main)]
pub fn to_string<T: Serialize>(t: &T) -> String {
    t.serialize()
}
