#[cfg(feature = "sets")]
#[cfg(test)]
mod test {
    extern crate alloc;

    use alloc::collections::BTreeSet;
    use core::str::FromStr;

    use edn::{List, Vector};
    use edn_rs::{edn, from_edn, from_str, hset, map, set, Edn, EdnError, Map, Set};

    fn err_as_string(s: &str) -> String {
        let err = Edn::from_str(s).err().unwrap();
        format!("{err:?}")
    }

    #[test]
    fn parse_set_with_commas() {
        assert_eq!(
            Edn::from_str("#{true, \\c, 3,four, }").unwrap(),
            Edn::Set(Set::new(set![
                Edn::Symbol("four".to_string()),
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3),
            ]))
        );
    }

    #[test]
    fn parse_comment_in_set() {
        assert_eq!(
            Edn::from_str("#{true ; bool true in a set\n \\c 3 }").unwrap(),
            Edn::Set(Set::new(set![
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3)
            ]))
        );
    }

    #[test]
    fn parse_true_false_nil_with_comments_in_set() {
        assert_eq!(
            Edn::from_str("#{true;this is true\nfalse;this is false\nnil;this is nil\n}").unwrap(),
            Edn::Set(Set::new(set![Edn::Bool(true), Edn::Bool(false), Edn::Nil,]))
        );
    }

    #[test]
    fn parse_comment_in_set_end() {
        assert_eq!(
            Edn::from_str("#{true \\c 3; int 3 in a set\n}").unwrap(),
            Edn::Set(Set::new(set![
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3)
            ]))
        );
    }

    #[test]
    fn from_str_list_with_set() {
        let edn = "(1 -10 \"2\" 3.3 :b #{true \\c})";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Int(-10),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Set(Set::new(set![Edn::Bool(true), Edn::Char('c')]))
            ]))
        );
    }

    #[test]
    fn parse_complex() {
        assert_eq!(
            Edn::from_str("[:b ( 5 \\c #{true \\c 3 } ) ]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Key(":b".to_string()),
                Edn::List(List::new(vec![
                    Edn::UInt(5),
                    Edn::Char('c'),
                    Edn::Set(Set::new(set![
                        Edn::Bool(true),
                        Edn::Char('c'),
                        Edn::UInt(3)
                    ]))
                ]))
            ]))
        );
    }

    #[test]
    fn parse_comment_complex() {
        assert_eq!(
            Edn::from_str("[:b ( 5 \\c #{true \\c; char c in a set\n3 } ) ]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Key(":b".to_string()),
                Edn::List(List::new(vec![
                    Edn::UInt(5),
                    Edn::Char('c'),
                    Edn::Set(Set::new(set![
                        Edn::Bool(true),
                        Edn::Char('c'),
                        Edn::UInt(3)
                    ]))
                ]))
            ]))
        );
    }

    #[test]
    fn from_str_complex_map() {
        let edn = "{:a \"2\" :b [true false] :c #{:A {:a :b} nil}}";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::Map(Map::new(map! {
            ":a".to_string() =>Edn::Str("2".to_string()),
            ":b".to_string() => Edn::Vector(Vector::new(vec![Edn::Bool(true), Edn::Bool(false)])),
            ":c".to_string() => Edn::Set(Set::new(
                set!{
                    Edn::Map(Map::new(map!{":a".to_string() => Edn::Key(":b".to_string())})),
                    Edn::Key(":A".to_string()),
                    Edn::Nil}))}))
        );
    }

    #[test]
    fn edn_element_with_inst() {
        assert_eq!(
            Edn::from_str(
                "#{ :a :b {:c :d :date  #inst \"2020-07-16T21:53:14.628-00:00\" ::c ::d} nil}"
            )
            .unwrap(),
            Edn::Set(Set::new(set! {
                Edn::Key(":a".to_string()),
                Edn::Key(":b".to_string()),
                Edn::Map(Map::new(map! {
                    ":c".to_string() => Edn::Key(":d".to_string()),
                    ":date".to_string() => Edn::Tagged("inst".to_string(), Box::new(Edn::Str("2020-07-16T21:53:14.628-00:00".to_string()))),
                    "::c".to_string() => Edn::Key("::d".to_string())
                })),
                Edn::Nil
            }))
        );
    }

    #[test]
    fn parse_discard_space_invalid() {
        assert_eq!(
            err_as_string(
                "#_ ,, #{hello, this will be discarded} #_{so will this} #{this is invalid"
            ),
            "EdnError { code: UnexpectedEOF, line: Some(1), column: Some(74), ptr: Some(73) }"
        );
    }

    #[test]
    fn parse_tagged_set() {
        assert_eq!(
            Edn::from_str("#domain/model #{1 2 3}").unwrap(),
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Set(Set::new(set![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        );
    }

    #[test]
    fn deser_btreeset_with_error() {
        let edn = "#{\"a\", 5, \"b\"}";
        let err: Result<BTreeSet<u64>, EdnError> = from_str(edn);
        assert!(err.is_err());
    }

    #[test]
    fn test_more_sym() {
        let edn: Edn = Edn::from_str("(a \\b \"c\" 5 #{hello world})").unwrap();
        let expected = Edn::List(List::new(vec![
            Edn::Symbol("a".to_string()),
            Edn::Char('b'),
            Edn::Str("c".to_string()),
            Edn::UInt(5u64),
            Edn::Set(Set::new(
                set! { Edn::Symbol("hello".to_string()), Edn::Symbol("world".to_string()) },
            )),
        ]));
        assert_eq!(edn, expected);
    }

    #[test]
    fn deser_btreeset() {
        let set = Edn::Set(Set::new(set! {
            Edn::UInt(4),
            Edn::UInt(5),
            Edn::UInt(6)
        }));
        let expected = set! {
            4,
            5,
            6,
        };
        let deser_set: std::collections::BTreeSet<u64> = from_edn(&set).unwrap();
        assert_eq!(deser_set, expected);
    }

    #[test]
    #[cfg(feature = "std")]
    fn deser_hashset() {
        use ordered_float::OrderedFloat;

        let set = Edn::Set(Set::new(set! {
            Edn::Double(4.6.into()),
            Edn::Double(5.6.into()),
            Edn::Double(6.6.into())
        }));
        let expected = hset! {
            OrderedFloat(4.6f64),
            OrderedFloat(5.6f64),
            OrderedFloat(6.6f64),
        };
        let deser_set: std::collections::HashSet<OrderedFloat<f64>> = from_edn(&set).unwrap();
        assert_eq!(deser_set, expected);
    }

    #[test]
    fn string_with_empty_set() {
        assert_eq!("\"#{}\"", format!("{}", Edn::from_str("\"#{}\"").unwrap()));
    }
}
