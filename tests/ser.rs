#![recursion_limit = "512"]

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

    use edn_derive::Serialize;
    use edn_rs::{hmap, hset, map, set};

    #[test]
    fn serializes_a_complex_structure() {
        #[derive(Serialize, Debug, Clone)]
        struct Example {
            btreemap: BTreeMap<String, Vec<String>>,
            btreeset: BTreeSet<i64>,
            hashmap: HashMap<String, Vec<String>>,
            hashset: HashSet<i64>,
            tuples: (i32, bool, char),
        }

        let edn = Example {
            btreemap: map! {"this is a key".to_string() => vec!["with".to_string(), "many".to_string(), "keys".to_string()]},
            btreeset: set! {3i64, 4i64, 5i64},
            hashmap: hmap! {"this is a key".to_string() => vec!["with".to_string(), "many".to_string(), "keys".to_string()]},
            hashset: hset! {3i64},
            tuples: (3i32, true, 'd'),
        };

        assert_eq!(edn_rs::to_string(&edn), "{ :btreemap {:this-is-a-key [\"with\", \"many\", \"keys\"]}, :btreeset #{3, 4, 5}, :hashmap {:this-is-a-key [\"with\", \"many\", \"keys\"]}, :hashset #{3}, :tuples (3, true, \\d), }");
    }

    #[test]
    fn serializes_nested_structures() {
        #[derive(Serialize, Debug, Clone)]
        struct Foo {
            value: bool,
        }

        #[derive(Serialize, Debug, Clone)]
        struct Bar {
            value: String,
            foo_vec: Vec<Foo>,
        }

        #[derive(Serialize, Debug, Clone)]
        struct FooBar {
            value: f64,
            bar: Bar,
        }

        let edn = FooBar {
            value: 3.4,
            bar: Bar {
                value: "data".to_string(),
                foo_vec: vec![Foo { value: false }, Foo { value: true }],
            },
        };

        assert_eq!(edn_rs::to_string(&edn), "{ :value 3.4, :bar { :value \"data\", :foo-vec [{ :value false, }, { :value true, }], }, }");
    }
}

#[test]
fn pub_struct() {
    let edn = helper::Edn {
        val: 6i32,
        tuples: (3i32, true, 'd'),
    };

    assert_eq!(edn.val, 6i32);
    assert_eq!(edn.tuples, (3i32, true, 'd'));
}

mod helper {
    use edn_derive::Serialize;

    #[derive(Serialize, Debug, Clone)]
    pub struct Edn {
        pub val: i32,
        pub tuples: (i32, bool, char),
    }
}
