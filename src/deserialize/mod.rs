use crate::edn::{Edn, Error};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::str::FromStr;

pub(crate) mod parse;

/// public trait to be used to `Deserialize` structs
///
/// Example:
/// ```
/// use crate::edn_rs::{Edn, EdnError, Deserialize};
///
/// #[derive(Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: usize,
/// }
///
/// impl Deserialize for Person {
///     fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
///         Ok(Self {
///             name: edn_rs::from_edn(&edn[":name"])?,
///             age: edn_rs::from_edn(&edn[":age"])?,
///         })
///     }
/// }
///
/// let edn_str = "{:name \"rose\" :age 66 }";
/// let person: Person = edn_rs::from_str(edn_str).unwrap();
///
/// assert_eq!(
///     person,
///     Person {
///         name: "rose".to_string(),
///         age: 66,
///     }
/// );
///
/// println!("{:?}", person);
/// // Person { name: "rose", age: 66 }
///
/// let bad_edn_str = "{:name \"rose\" :age \"some text\" }";
/// let person: Result<Person, EdnError> = edn_rs::from_str(bad_edn_str);
///
/// assert_eq!(
///     person,
///     Err(EdnError::Deserialize(
///         "couldn't convert `\"some text\"` into `uint`".to_string()
///     ))
/// );
/// ```
pub trait Deserialize: Sized {
    fn deserialize(edn: &Edn) -> Result<Self, Error>;
}

fn build_deserialize_error(edn: &Edn, type_: &str) -> Error {
    Error::Deserialize(format!("couldn't convert `{}` into `{}`", edn, type_))
}

impl Deserialize for () {
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Nil => Ok(()),
            _ => Err(build_deserialize_error(&edn, "unit")),
        }
    }
}

macro_rules! impl_deserialize_float {
    ( $( $name:ty ),+ ) => {
        $(
            impl Deserialize for $name
            {
                fn deserialize(edn: &Edn) -> Result<Self, Error> {
                    edn
                        .to_float()
                        .ok_or_else(|| build_deserialize_error(&edn, "float"))
                        .map(|u| u as $name)
                }
            }
        )+
    };
}

impl_deserialize_float!(f32, f64);

macro_rules! impl_deserialize_int {
    ( $( $name:ty ),+ ) => {
        $(
            impl Deserialize for $name
            {
                fn deserialize(edn: &Edn) -> Result<Self, Error> {
                    edn
                        .to_int()
                        .ok_or_else(|| build_deserialize_error(&edn, "int"))
                        .map(|u| u as $name)
                }
            }
        )+
    };
}

impl_deserialize_int!(isize, i8, i16, i32, i64);

macro_rules! impl_deserialize_uint {
    ( $( $name:ty ),+ ) => {
        $(
            impl Deserialize for $name
            {
                fn deserialize(edn: &Edn) -> Result<Self, Error> {
                    edn
                        .to_uint()
                        .ok_or_else(|| build_deserialize_error(&edn, "uint"))
                        .map(|u| u as $name)
                }
            }
        )+
    };
}

impl_deserialize_uint!(usize, u8, u16, u32, u64);

impl Deserialize for bool {
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        edn.to_bool()
            .ok_or_else(|| build_deserialize_error(&edn, "bool"))
    }
}

impl Deserialize for String {
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Str(s) => {
                if s.starts_with('\"') {
                    Ok(s.replace('\"', ""))
                } else {
                    Ok(s.to_owned())
                }
            }
            e => Ok(e.to_string()),
        }
    }
}

impl Deserialize for char {
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        edn.to_char()
            .ok_or_else(|| build_deserialize_error(&edn, "char"))
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Vector(_) => Ok(edn
                .iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|e| Deserialize::deserialize(e))
                .collect::<Result<Vec<T>, Error>>()?),
            Edn::List(_) => Ok(edn
                .iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|e| Deserialize::deserialize(e))
                .collect::<Result<Vec<T>, Error>>()?),
            Edn::Set(_) => Ok(edn
                .iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|e| Deserialize::deserialize(e))
                .collect::<Result<Vec<T>, Error>>()?),
            _ => Err(build_deserialize_error(
                &edn,
                std::any::type_name::<Vec<T>>(),
            )),
        }
    }
}

impl<T> Deserialize for HashMap<String, T>
where
    T: Deserialize,
{
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Map(_) => Ok(edn
                .map_iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|(key, e)| (key, Deserialize::deserialize(e).unwrap()))
                .fold(HashMap::new(), |mut acc, (key, e)| {
                    acc.insert(key.to_string(), e);
                    acc
                })),
            Edn::NamespacedMap(ns, _) => Ok(edn
                .map_iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|(key, e)| {
                    (
                        ns.to_string() + "/" + key,
                        Deserialize::deserialize(e).unwrap(),
                    )
                })
                .fold(HashMap::new(), |mut acc, (key, e)| {
                    acc.insert(key.to_string(), e);
                    acc
                })),
            _ => Err(build_deserialize_error(
                &edn,
                std::any::type_name::<HashMap<String, T>>(),
            )),
        }
    }
}

impl<T> Deserialize for BTreeMap<String, T>
where
    T: Deserialize,
{
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Map(_) => Ok(edn
                .map_iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|(key, e)| (key, Deserialize::deserialize(e).unwrap()))
                .fold(BTreeMap::new(), |mut acc, (key, e)| {
                    acc.insert(key.to_string(), e);
                    acc
                })),
            Edn::NamespacedMap(ns, _) => Ok(edn
                .map_iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|(key, e)| {
                    (
                        ns.to_string() + "/" + key,
                        Deserialize::deserialize(e).unwrap(),
                    )
                })
                .fold(BTreeMap::new(), |mut acc, (key, e)| {
                    acc.insert(key.to_string(), e);
                    acc
                })),
            _ => Err(build_deserialize_error(
                &edn,
                std::any::type_name::<BTreeMap<String, T>>(),
            )),
        }
    }
}

impl<T: std::cmp::Eq + std::hash::Hash> Deserialize for HashSet<T>
where
    T: Deserialize,
{
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Set(_) => Ok(edn
                .set_iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|e| Deserialize::deserialize(e).unwrap())
                .fold(HashSet::new(), |mut acc, e| {
                    acc.insert(e);
                    acc
                })),
            _ => Err(build_deserialize_error(
                &edn,
                std::any::type_name::<HashSet<T>>(),
            )),
        }
    }
}

impl<T: std::cmp::Eq + std::hash::Hash + std::cmp::Ord> Deserialize for BTreeSet<T>
where
    T: Deserialize,
{
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Set(_) => Ok(edn
                .set_iter()
                .ok_or_else(|| Error::Iter(format!("Could not create iter from {:?}", edn)))?
                .map(|e| Deserialize::deserialize(e).unwrap())
                .fold(BTreeSet::new(), |mut acc, e| {
                    acc.insert(e);
                    acc
                })),
            _ => Err(build_deserialize_error(
                &edn,
                std::any::type_name::<BTreeSet<T>>(),
            )),
        }
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    fn deserialize(edn: &Edn) -> Result<Self, Error> {
        match edn {
            Edn::Nil => Ok(None),
            _ => Ok(Some(from_edn(&edn)?)),
        }
    }
}

/// `from_str` deserializes an EDN String into type `T` that implements `Deserialize`. Response is `Result<T, EdnError>`
///
/// ```
/// use edn_rs::{Deserialize, Edn, EdnError};
///
/// #[derive(Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: usize,
/// }
///
/// impl Deserialize for Person {
///     fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
///         Ok(Self {
///             name: edn_rs::from_edn(&edn[":name"])?,
///             age: edn_rs::from_edn(&edn[":age"])?,
///         })
///     }
/// }
///
/// let edn_str = "  {:name \"rose\" :age 66  }  ";
/// let person: Person = edn_rs::from_str(edn_str).unwrap();
///
/// println!("{:?}", person);
/// // Person { name: "rose", age: 66 }
///
/// assert_eq!(
///     person,
///     Person {
///         name: "rose".to_string(),
///         age: 66,
///     }
/// );
///
/// let bad_edn_str = "{:name \"rose\" :age \"some text\" }";
/// let person: Result<Person, EdnError> = edn_rs::from_str(bad_edn_str);
///
/// assert_eq!(
///     person,
///     Err(EdnError::Deserialize(
///             "couldn't convert `\"some text\"` into `uint`".to_string()
///     ))
/// );
/// ```
pub fn from_str<T: Deserialize>(s: &str) -> Result<T, Error> {
    let edn = Edn::from_str(s)?;
    from_edn(&edn)
}

/// `from_edn` deserializes an EDN type into a `T` type that implements `Deserialize`. Response is `Result<T, EdnError>`
///
/// ```
/// use edn_rs::{map, Deserialize, Edn, EdnError, Map};
///
/// #[derive(Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: usize,
/// }
///
/// impl Deserialize for Person {
///     fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
///         Ok(Self {
///             name: edn_rs::from_edn(&edn[":name"])?,
///             age: edn_rs::from_edn(&edn[":age"])?,
///         })
///     }
/// }
///
/// let edn = Edn::Map(Map::new(map! {
///     ":name".to_string() => Edn::Str("rose".to_string()),
///     ":age".to_string() => Edn::UInt(66)
/// }));
/// let person: Person = edn_rs::from_edn(&edn).unwrap();
///
/// println!("{:?}", person);
/// // Person { name: "rose", age: 66 }
///
/// assert_eq!(
///     person,
///     Person {
///         name: "rose".to_string(),
///         age: 66,
///     }
/// );
///
/// let bad_edn = Edn::Map(Map::new(map! {
///     ":name".to_string() => Edn::Str("rose".to_string()),
///     ":age".to_string() => Edn::Str("some text".to_string())
/// }));
/// let person: Result<Person, EdnError> = edn_rs::from_edn(&bad_edn);
///
/// assert_eq!(
///     person,
///     Err(EdnError::Deserialize(
///         "couldn't convert `\"some text\"` into `uint`".to_string()
///     ))
/// );
/// ```
pub fn from_edn<T: Deserialize>(edn: &Edn) -> Result<T, Error> {
    T::deserialize(edn)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::edn::{List, Map, Set, Vector};
    use crate::{map, set};

    #[test]
    fn unit() {
        let nil = "nil";
        let unit: () = from_str(nil).unwrap();

        assert_eq!(unit, ())
    }

    #[test]
    fn from_str_simple_vec() {
        let edn = "[1 \"2\" 3.3 :b true \\c]";

        assert_eq!(
            Edn::from_str(edn),
            Ok(Edn::Vector(Vector::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Bool(true),
                Edn::Char('c')
            ])))
        );
    }

    #[test]
    fn from_str_list_with_vec() {
        let edn = "(1 \"2\" 3.3 :b [true \\c])";

        assert_eq!(
            Edn::from_str(edn),
            Ok(Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Vector(Vector::new(vec![Edn::Bool(true), Edn::Char('c')]))
            ])))
        );
    }

    #[test]
    fn from_str_list_with_set() {
        let edn = "(1 -10 \"2\" 3.3 :b #{true \\c})";

        assert_eq!(
            Edn::from_str(edn),
            Ok(Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Int(-10),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Set(Set::new(set![Edn::Bool(true), Edn::Char('c')]))
            ])))
        );
    }

    #[test]
    fn from_str_simple_map() {
        let edn = "{:a \"2\" :b true :c nil}";

        assert_eq!(
            Edn::from_str(edn),
            Ok(Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("2".to_string()),
                ":b".to_string() => Edn::Bool(true), ":c".to_string() => Edn::Nil}
            )))
        );
    }

    #[test]
    fn from_str_complex_map() {
        let edn = "{:a \"2\" :b [true false] :c #{:A {:a :b} nil}}";

        assert_eq!(
            Edn::from_str(edn),
            Ok(Edn::Map(Map::new(map! {
            ":a".to_string() =>Edn::Str("2".to_string()),
            ":b".to_string() => Edn::Vector(Vector::new(vec![Edn::Bool(true), Edn::Bool(false)])),
            ":c".to_string() => Edn::Set(Set::new(
                set!{
                    Edn::Map(Map::new(map!{":a".to_string() => Edn::Key(":b".to_string())})),
                    Edn::Key(":A".to_string()),
                    Edn::Nil}))})))
        );
    }

    #[test]
    fn from_str_wordy_str() {
        let edn = "[\"hello brave new world\"]";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::Vector(Vector::new(vec![Edn::Str(
                "hello brave new world".to_string()
            )]))
        )
    }

    #[test]
    fn namespaced_maps() {
        let edn = ":abc{ 0 :val 1 :value}";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::NamespacedMap(
                "abc".to_string(),
                Map::new(map! {
                    "0".to_string() => Edn::Key(":val".to_string()),
                    "1".to_string() => Edn::Key(":value".to_string())
                })
            )
        );
    }

    #[test]
    fn uuid() {
        let uuid = "#uuid \"af6d8699-f442-4dfd-8b26-37d80543186b\"";
        let edn: Edn = Edn::from_str(uuid).unwrap();

        assert_eq!(
            edn,
            Edn::Uuid("af6d8699-f442-4dfd-8b26-37d80543186b".to_string())
        )
    }

    #[test]
    fn deserialize_struct_with_vec() {
        #[derive(PartialEq, Debug)]
        struct Foo {
            bar: Vec<Option<usize>>,
        }
        impl Deserialize for Foo {
            fn deserialize(edn: &Edn) -> Result<Self, Error> {
                Ok(Foo {
                    bar: from_edn(&edn[":bar"])?,
                })
            }
        }
        let edn_foo = "{:bar [1 nil 3]}";
        let foo: Foo = from_str(edn_foo).unwrap();

        assert_eq!(
            foo,
            Foo {
                bar: vec![Some(1), None, Some(3)],
            }
        );
    }

    #[test]
    fn test_sym() {
        let edn: Edn = Edn::from_str("(a b c your-hair!-is+_parsed?)").unwrap();
        let expected = Edn::List(List::new(vec![
            Edn::Symbol("a".to_string()),
            Edn::Symbol("b".to_string()),
            Edn::Symbol("c".to_string()),
            Edn::Symbol("your-hair!-is+_parsed?".to_string()),
        ]));
        assert_eq!(edn, expected);
    }

    #[test]
    fn test_more_sym() {
        let edn: Edn = Edn::from_str("(a \\b \"c\" 5 #{hello world})").unwrap();
        let expected = Edn::List(List::new(vec![
            Edn::Symbol("a".to_string()),
            Edn::Char('b'),
            Edn::Str("c".to_string()),
            Edn::UInt(5usize),
            Edn::Set(Set::new(
                set! { Edn::Symbol("hello".to_string()), Edn::Symbol("world".to_string()) },
            )),
        ]));
        assert_eq!(edn, expected);
    }

    #[test]
    fn namespaced_maps_navigation() {
        let edn_str = ":abc{ 0 :val 1 :value}";

        let edn = Edn::from_str(edn_str).unwrap();

        assert_eq!(edn[0], Edn::Key(":val".to_string()));
        assert_eq!(edn["0"], Edn::Key(":val".to_string()));
        assert_eq!(edn[1], Edn::Key(":value".to_string()));
        assert_eq!(edn["1"], Edn::Key(":value".to_string()));
    }
}
