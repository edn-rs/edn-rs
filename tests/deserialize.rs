#[cfg(test)]
mod test {
    extern crate alloc;

    use alloc::collections::BTreeMap;
    use core::str::FromStr;

    use edn_rs::{edn, from_edn, from_str, hmap, map, Edn, List, Map, Vector};

    fn err_as_string(s: &str) -> String {
        let err = Edn::from_str(s).err().unwrap();
        format!("{err:?}")
    }

    #[test]
    fn unit() {
        let nil = "nil";
        let unit: () = from_str(nil).unwrap();

        assert_eq!(unit, ());
    }

    #[test]
    fn parse_empty() {
        assert_eq!(Edn::from_str("").unwrap(), Edn::Empty);
        assert_eq!(
            Edn::from_str("[]").unwrap(),
            Edn::Vector(Vector::new(vec![]))
        );
        assert_eq!(Edn::from_str("()").unwrap(), Edn::List(List::new(vec![])));
    }

    #[test]
    fn parse_whitespace_only() {
        let edn = "
                          \r\n";

        assert_eq!(Edn::from_str(edn).unwrap(), Edn::Empty);
    }

    #[test]
    #[cfg(not(feature = "sets"))]
    // Special case of running into a set without the feature enabled
    fn parse_set_without_set_feature() {
        assert!(Edn::from_str("#{true, \\c, 3,four, }").is_err())
    }

    #[test]
    fn parse_commas_are_whitespace() {
        assert_eq!(Edn::from_str(",,,,, \r\n,,,").unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_keyword() {
        assert_eq!(
            Edn::from_str(":keyword").unwrap(),
            Edn::Key(":keyword".to_string())
        );
    }

    #[test]
    fn parse_str() {
        assert_eq!(
            Edn::from_str("\"hello world, from      RUST\"").unwrap(),
            Edn::Str("hello world, from      RUST".to_string())
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
        );
    }

    #[test]
    fn parse_str_top_level_comment() {
        assert_eq!(
            Edn::from_str(";;; hello world string example\n\n;; deserialize the following string\n\n\"hello world, from      RUST\"").unwrap(),
            Edn::Str("hello world, from      RUST".to_string())
        );
    }

    #[test]
    fn parse_str_top_level_comment_whitespace() {
        assert_eq!(
            Edn::from_str("\n;;; hello world string example\n\n;; deserialize the following string\n\n,,\"hello world, from      RUST\"").unwrap(),
            Edn::Str("hello world, from      RUST".to_string())
        );
    }

    #[test]
    fn parse_str_looks_like_comment() {
        assert_eq!(
            Edn::from_str("\";;; hello world, from      RUST\n\"").unwrap(),
            Edn::Str(";;; hello world, from      RUST\n".to_string())
        );
    }

    #[test]
    fn parse_str_with_escaped_characters() {
        assert_eq!(
            Edn::from_str(r#""hello\n \r \t \"world\" with escaped \\ characters""#).unwrap(),
            Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())
        );
    }

    #[test]
    fn parse_str_with_invalid_escape() {
        assert_eq!(
            err_as_string(r#""hello\n \r \t \"world\" with escaped \\ \g characters""#),
            "EdnError { code: InvalidEscape, line: Some(1), column: Some(44), ptr: Some(43) }"
        );
    }

    #[test]
    fn parse_unterminated_string() {
        assert_eq!(
            err_as_string(r#""hello\n \r \t \"world\" with escaped \\ characters"#),
            "EdnError { code: UnexpectedEOF, line: Some(1), column: Some(52), ptr: Some(51) }"
        );
    }

    #[test]
    fn parse_number() {
        assert_eq!(Edn::from_str("7").unwrap(), Edn::UInt(7));
        assert_eq!(Edn::from_str("143").unwrap(), Edn::UInt(143));
        assert_eq!(Edn::from_str("-435143").unwrap(), Edn::Int(-435_143));
        assert_eq!(
            Edn::from_str("-43.5143").unwrap(),
            Edn::Double(edn::Double::from(-43.5143))
        );
        assert_eq!(Edn::from_str("43/5143").unwrap(), Edn::Rational((43, 5143)));
        assert_eq!(
            Edn::from_str("999999999999999999999.0").unwrap(),
            Edn::Double(edn::Double::from(1e21f64))
        );
    }

    #[test]
    fn parse_char() {
        assert_eq!(Edn::from_str("\\k").unwrap(), Edn::Char('k'));
    }

    #[test]
    fn parse_bool_or_nil() {
        assert_eq!(Edn::from_str("true").unwrap(), Edn::Bool(true));
        assert_eq!(Edn::from_str("false").unwrap(), Edn::Bool(false));
        assert_eq!(Edn::from_str("nil").unwrap(), Edn::Nil);
        assert_eq!(
            Edn::from_str("\"true\"").unwrap(),
            Edn::Str("true".to_string())
        );
    }

    #[test]
    fn from_str_simple_vec() {
        let edn = "[1 \"2\" 3.3 :b true \\c]";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Bool(true),
                Edn::Char('c')
            ]))
        );
    }

    #[test]
    fn parse_simple_vec() {
        assert_eq!(
            Edn::from_str("[11 \"2\" 3.3 :b true \\c]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::UInt(11),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Bool(true),
                Edn::Char('c')
            ]))
        );
    }

    #[test]
    fn parse_comment_in_simple_vec() {
        assert_eq!(
            Edn::from_str("[11 \"2\" 3.3 ; float in simple vec\n:b true \\c]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::UInt(11),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Bool(true),
                Edn::Char('c')
            ]))
        );
    }

    #[test]
    fn parse_comment_in_simple_vec_end() {
        assert_eq!(
            Edn::from_str("[11 \"2\" 3.3 :b true \\c; char in simple vec\n]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::UInt(11),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Bool(true),
                Edn::Char('c')
            ]))
        );
    }

    #[test]
    fn parse_comment_in_simple_vec_str_literal() {
        let edn = "[
                         11
                        \"2\"
                         3.3
                         ;; float in simple vec
                         :b
                         true
                         \\c
                       ]";
        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::UInt(11),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Bool(true),
                Edn::Char('c')
            ]))
        );
    }

    #[test]
    fn parse_bool_in_newline_simple_vec_str_literal() {
        assert_eq!(
            Edn::from_str("[\ntrue\nfalse\nnil\n]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Bool(true),
                Edn::Bool(false),
                Edn::Nil,
            ]))
        );
    }

    #[test]
    fn parse_bool_in_tab_simple_vec_str_literal() {
        assert_eq!(
            Edn::from_str("[\ttrue\tnil\tfalse\t]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Bool(true),
                Edn::Nil,
                Edn::Bool(false),
            ]))
        );
    }

    #[test]
    fn parse_bool_in_crlf_newline_simple_vec_str_literal() {
        assert_eq!(
            Edn::from_str("[\r\nnil\r\nfalse\r\ntrue\r\n]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Nil,
                Edn::Bool(false),
                Edn::Bool(true),
            ]))
        );
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            Edn::from_str("(1 \"2\" 3.3 :b )").unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
            ]))
        );
    }

    #[test]
    fn from_str_list_with_vec() {
        let edn = "(1 \"2\" 3.3 :b [true \\c])";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
                Edn::Vector(Vector::new(vec![Edn::Bool(true), Edn::Char('c')]))
            ]))
        );
    }

    #[test]
    fn parse_comment_in_list() {
        assert_eq!(
            Edn::from_str("(1 \"2\"; string in list\n3.3 :b )").unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
            ]))
        );
    }

    #[test]
    fn parse_comment_in_list_end() {
        assert_eq!(
            Edn::from_str("(1 \"2\" 3.3 :b; keyword in list\n)").unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
            ]))
        );
    }

    #[test]
    fn parse_simple_map() {
        assert_eq!(
            Edn::from_str("{:a \"2\" :b false :c nil }").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("2".to_string()),
                ":b".to_string() => Edn::Bool(false), ":c".to_string() => Edn::Nil}
            ))
        );
    }

    #[test]
    fn from_str_simple_map() {
        let edn = "{:a \"2\" :b true :c nil}";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("2".to_string()),
                ":b".to_string() => Edn::Bool(true), ":c".to_string() => Edn::Nil}
            ))
        );
    }

    #[test]
    fn deser_btreemap() {
        let ns_map = Edn::Map(Map::new(map! {
            ":a".to_string() => Edn::Vector(Vector::new(vec![Edn::Key(":val".to_string())])),
            ":b".to_string() => Edn::Vector(Vector::new(vec![Edn::Key(":value".to_string())]))
        }));
        let expected = map! {
            ":a".to_string() => vec![":val".to_string()],
            ":b".to_string() => vec![":value".to_string()]
        };
        let map: BTreeMap<String, Vec<String>> = from_edn(&ns_map).unwrap();
        assert_eq!(map, expected);
    }

    #[test]
    #[cfg(feature = "std")]
    fn deser_hashmap() {
        let ns_map = Edn::Map(Map::new(map! {
            ":a".to_string() => Edn::Bool(true),
            ":b".to_string() => Edn::Bool(false)
        }));
        let expected = hmap! {
            ":a".to_string() => true,
            ":b".to_string() => false
        };
        let map: std::collections::HashMap<String, bool> = from_edn(&ns_map).unwrap();
        assert_eq!(map, expected);
    }

    #[test]
    fn parse_inst() {
        assert_eq!(
            Edn::from_str("{:date  #inst \"2020-07-16T21:53:14.628-00:00\"}").unwrap(),
            Edn::Map(Map::new(map! {
                ":date".to_string() =>
                    Edn::Tagged("inst".to_string(),
                                Box::new(Edn::Str("2020-07-16T21:53:14.628-00:00".to_string())))
            }))
        );
    }

    #[test]
    fn uuid() {
        let uuid = "#uuid \"af6d8699-f442-4dfd-8b26-37d80543186b\"";
        let edn = Edn::from_str(uuid).unwrap();

        assert_eq!(
            edn,
            Edn::Tagged(
                "uuid".to_string(),
                Box::new(Edn::Str("af6d8699-f442-4dfd-8b26-37d80543186b".to_string()))
            )
        );
    }

    #[test]
    fn parse_tagged_int() {
        assert_eq!(
            Edn::from_str("#iasdf 234").unwrap(),
            Edn::Tagged(String::from("iasdf"), Box::new(Edn::UInt(234)))
        );
    }

    #[test]
    fn parse_discard_valid() {
        assert_eq!(Edn::from_str("#_iasdf 234").unwrap(), Edn::UInt(234));
    }

    #[test]
    fn parse_discard_invalid() {
        assert_eq!(
            err_as_string("#_{ 234"),
            "EdnError { code: UnexpectedEOF, line: Some(1), column: Some(8), ptr: Some(7) }"
        );
    }

    #[test]
    fn parse_discard_space_valid() {
        assert_eq!(Edn::from_str("#_ ,, 234 567").unwrap(), Edn::UInt(567));
    }

    #[test]
    fn parse_discard_empty() {
        assert_eq!(Edn::from_str("#_ ,, foo").unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_discard_repeat_empty() {
        assert_eq!(
            Edn::from_str("#_ ,, #_{discard again} #_ {:and :again} :okay").unwrap(),
            Edn::Empty
        );
    }

    #[test]
    fn parse_discard_repeat_not_empty() {
        assert_eq!(
            Edn::from_str("#_ ,, #_{discard again} #_ {:and :again} :okay {:a map}").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Symbol("map".to_string())}
            ))
        );
    }

    #[test]
    fn parse_discard_no_follow_element() {
        assert_eq!(
            err_as_string("#_ ,, "),
            "EdnError { code: UnexpectedEOF, line: Some(1), column: Some(7), ptr: Some(6) }"
        );
    }

    #[test]
    fn parse_discard_end_of_seq() {
        assert_eq!(
            Edn::from_str("[:foo #_ foo]").unwrap(),
            Edn::Vector(Vector::new(vec![Edn::Key(":foo".to_string())]))
        );
    }

    #[test]
    fn parse_discard_end_of_seq_no_follow() {
        assert_eq!(
            err_as_string("[:foo #_ ]"),
            "EdnError { code: UnexpectedEOF, line: Some(1), column: Some(10), ptr: Some(9) }"
        );
    }

    #[test]
    fn parse_discard_inside_seq() {
        assert_eq!(
            Edn::from_str("#_\"random comment\" [:a :b :c #_(:hello :world) :d]").unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Key(":a".to_string()),
                Edn::Key(":b".to_string()),
                Edn::Key(":c".to_string()),
                Edn::Key(":d".to_string())
            ]))
        );
    }

    #[test]
    fn parse_map_keyword_with_commas() {
        assert_eq!(
            Edn::from_str("{ :a :something, :b false, :c nil, }").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Key(":something".to_string()),
                ":b".to_string() => Edn::Bool(false), ":c".to_string() => Edn::Nil}
            ))
        );
    }

    #[test]
    fn parse_map_with_special_char_str1() {
        assert_eq!(
            Edn::from_str("{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }")
                .unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_only() {
        assert_eq!(
            Edn::from_str(" ;;; this is a comment\n").unwrap(),
            Edn::Empty
        );
    }

    #[test]
    fn parse_comment_only_no_newline() {
        assert_eq!(Edn::from_str(" ;;; this is a comment").unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_comment_multiple() {
        assert_eq!(
            Edn::from_str(" ;;; comment 1\n ;;; comment 2\n ;;; comment 3\n\n").unwrap(),
            Edn::Empty
        );
    }

    #[test]
    fn parse_comment_top_level() {
        assert_eq!(
            Edn::from_str(" ;; this is a map\n{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_inside_map() {
        assert_eq!(
            Edn::from_str("{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" ; escaped chars\n }").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_end_of_file() {
        assert_eq!(
            Edn::from_str(";; this is a map\n{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }\n ;; end of file\n").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_end_of_file_no_newline() {
        assert_eq!(
            Edn::from_str(";; this is a map\n{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }\n ;; end of file").unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_tagged_vec() {
        assert_eq!(
            Edn::from_str("#domain/model [1 2 3]").unwrap(),
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Vector(Vector::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        );
    }

    #[test]
    fn parse_tagged_vec_with_comment() {
        assert_eq!(
            Edn::from_str("#domain/model ; tagging this vector\n [1 2 3]").unwrap(),
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Vector(Vector::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        );
    }

    #[test]
    fn parse_map_with_tagged_vec() {
        assert_eq!(
            Edn::from_str("{ :model #domain/model [1 2 3] :int 2 }").unwrap(),
            Edn::Map(Map::new(map! {
                ":int".to_string() => Edn::UInt(2),
                ":model".to_string() => Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Vector(Vector::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )}))
        );
    }

    #[test]
    fn parse_tagged_list() {
        assert_eq!(
            Edn::from_str("#domain/model (1 2 3)").unwrap(),
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::List(List::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        );
    }

    #[test]
    fn parse_tagged_str() {
        assert_eq!(
            Edn::from_str("#domain/model \"hello\"").unwrap(),
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Str(String::from("hello")))
            )
        );
    }

    #[test]
    fn parse_tagged_map() {
        assert_eq!(
            Edn::from_str("#domain/model {1 2 3 4}").unwrap(),
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Map(Map::new(map! {
                    "1".to_string() =>
                    Edn::UInt(2),
                    "3".to_string() =>
                    Edn::UInt(4)
                })))
            )
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

        let lorem = "Lorem-ipsum-dolor-sit-amet-consectetur-adipiscing-elit-sed-do-eiusmod-tempor-incididunt-ut-labore-et-dolore-magna-aliqua.-Ut-enim-ad-minim-veniam-quis-nostrud-exercitation-ullamco-laboris-nisi-ut-aliquip-ex-ea-commodo-consequat.-Duis-aute-irure-dolor-in-reprehenderit-in-voluptate-velit-esse-cillum-dolore-eu-fugiat-nulla-pariatur.-Excepteur-sint-occaecat-cupidatat-non-proident-sunt-in-culpa-qui-officia-deserunt-mollit-anim-id-est-laborum".to_string();
        assert_eq!(Edn::from_str(lorem.as_str()).unwrap(), Edn::Symbol(lorem));
    }

    #[test]
    fn test_nft() {
        let t: Edn = Edn::from_str("tTEST").unwrap();
        let f: Edn = Edn::from_str("fTEST").unwrap();
        let n: Edn = Edn::from_str("nTEST").unwrap();
        let err: Edn = Edn::from_str("fTE").unwrap();

        assert_eq!(n, Edn::Symbol("nTEST".to_string()));
        assert_eq!(f, Edn::Symbol("fTEST".to_string()));
        assert_eq!(t, Edn::Symbol("tTEST".to_string()));
        assert_eq!(err, Edn::Symbol("fTE".to_string()));
    }

    #[test]
    fn parse_tagged_map_anything() {
        let edn = "#domain/model \n;; cool a tagged map!!!\n {1 \"hello\" 3 [[1 2] [2 3] [3,, 4]] #keyword, :4,,, {:cool-tagged #yay ;; what a tag inside a tagged map?!\n {:stuff \"hehe\"}}, 5 #wow {:a, :b}}";
        let res = Edn::from_str(edn).unwrap();

        println!("{res:#?}\n\n");

        assert_eq!(
            res,
            Edn::Tagged(
                "domain/model".to_string(),
                Box::new(Edn::Map(Map::new(map! {
                    "#keyword :4".to_string() => Edn::Map(
                        Map::new(map!
                            {
                                ":cool-tagged".to_string() => Edn::Tagged(
                                    "yay".to_string(),
                                    Box::new(Edn::Map(
                                        Map::new(
                                            map!{
                                                ":stuff".to_string() => Edn::Str(
                                                    "hehe".to_string(),
                                                )
                                            },
                                        ),
                                    )),
                                )
                            },
                        ),
                    ),
                    "1".to_string() => Edn::Str(
                        "hello".to_string(),
                    ),
                    "3".to_string() => Edn::Vector(
                        Vector::new(
                            vec![
                                Edn::Vector(
                                    Vector::new(
                                        vec![
                                            Edn::UInt(
                                                1,
                                            ),
                                            Edn::UInt(
                                                2,
                                            ),
                                        ],
                                    ),
                                ),
                                Edn::Vector(
                                    Vector::new(
                                        vec![
                                            Edn::UInt(
                                                2,
                                            ),
                                            Edn::UInt(
                                                3,
                                            ),
                                        ],
                                    ),
                                ),
                                Edn::Vector(
                                    Vector::new(
                                        vec![
                                            Edn::UInt(
                                                3,
                                            ),
                                            Edn::UInt(
                                                4,
                                            ),
                                        ],
                                    ),
                                ),
                            ],
                        ),
                    ),
                    "5".to_string() => Edn::Tagged(
                        "wow".to_string(),
                        Box::new(Edn::Map(
                            Map::new(map!
                                {
                                    ":a".to_string() => Edn::Key(
                                        ":b".to_string(),
                                    )
                                },
                            ),
                        )),
                    )
                },),),)
            )
        );
    }

    #[test]
    fn parse_exp() {
        assert_eq!(
            Edn::from_str("5.01122771367421E15").unwrap(),
            Edn::Double(5_011_227_713_674_210_f64.into())
        );
    }

    #[test]
    fn parse_numberic_symbol_with_doube_e() {
        assert!(Edn::from_str("5011227E71367421E12").is_err());
    }

    #[test]
    fn parse_exp_plus_sign() {
        assert_eq!(
            Edn::from_str("5.01122771367421E+12").unwrap(),
            Edn::Double(5_011_227_713_674.21_f64.into())
        );
    }

    #[test]
    fn parse_float_e_minus_12() {
        assert_eq!(
            Edn::from_str("0.00000000000501122771367421").unwrap(),
            Edn::Double(5.011_227_713_674_21e-12.into())
        );
    }

    #[test]
    fn parse_exp_minus_sign() {
        let res = Edn::from_str("5.01122771367421e-12").unwrap();
        assert_eq!(
            res,
            Edn::Double(0.000_000_000_005_011_227_713_674_21.into())
        );
        assert_eq!(res.to_string(), "0.00000000000501122771367421");
    }

    #[test]
    fn parse_0x_ints() {
        assert_eq!(Edn::from_str("0x2a").unwrap(), Edn::UInt(42));
        assert_eq!(Edn::from_str("-0X2A").unwrap(), Edn::Int(-42));
    }

    #[test]
    fn parse_radix_ints() {
        assert_eq!(Edn::from_str("16r2a").unwrap(), Edn::UInt(42));
        assert_eq!(Edn::from_str("8r63").unwrap(), Edn::UInt(51));
        assert_eq!(Edn::from_str("36rabcxyz").unwrap(), Edn::UInt(623_741_435));
        assert_eq!(Edn::from_str("-16r2a").unwrap(), Edn::Int(-42));
        assert_eq!(Edn::from_str("-32rFOObar").unwrap(), Edn::Int(-529_280_347));
    }

    #[test]
    fn parse_invalid_ints() {
        assert_eq!(
            err_as_string("42invalid123"),
            "EdnError { code: InvalidNumber, line: Some(1), column: Some(1), ptr: Some(0) }"
        );
        assert_eq!(
            err_as_string("0xxyz123"),
            "EdnError { code: InvalidNumber, line: Some(1), column: Some(1), ptr: Some(0) }"
        );
        assert_eq!(err_as_string("42rabcxzy"), "EdnError { code: InvalidRadix(Some(42)), line: Some(1), column: Some(1), ptr: Some(0) }");
        assert_eq!(
            err_as_string("42crazyrabcxzy"),
            "EdnError { code: InvalidRadix(None), line: Some(1), column: Some(1), ptr: Some(0) }"
        );
    }

    #[test]
    fn leading_plus_symbol_int() {
        assert_eq!(Edn::from_str("+42").unwrap(), Edn::UInt(42));
        assert_eq!(Edn::from_str("+0x2a").unwrap(), Edn::UInt(42));
    }

    #[test]
    fn lisp_quoted() {
        assert_eq!(
            Edn::from_str("('(symbol))").unwrap(),
            Edn::List(List::new(vec![
                Edn::Symbol("'".to_string()),
                Edn::List(List::new(vec![Edn::Symbol("symbol".to_string()),]))
            ]))
        );

        assert_eq!(
            Edn::from_str("(apply + '(1 2 3))").unwrap(),
            Edn::List(List::new(vec![
                Edn::Symbol("apply".to_string()),
                Edn::Symbol("+".to_string()),
                Edn::Symbol("'".to_string()),
                Edn::List(List::new(vec![Edn::UInt(1), Edn::UInt(2), Edn::UInt(3),]))
            ]))
        );

        assert_eq!(
            Edn::from_str("('(''symbol'foo''bar''))").unwrap(),
            Edn::List(List::new(vec![
                Edn::Symbol("'".to_string()),
                Edn::List(List::new(vec![Edn::Symbol(
                    "''symbol'foo''bar''".to_string()
                ),]))
            ]))
        );
    }

    #[test]
    fn minus_char_symbol() {
        assert_eq!(
            Edn::from_str("-foobar").unwrap(),
            Edn::Symbol("-foobar".to_string())
        );

        assert_eq!(
            Edn::from_str("(+foobar +foo+bar+ +'- '-+)").unwrap(),
            Edn::List(List::new(vec![
                Edn::Symbol("+foobar".to_string()),
                Edn::Symbol("+foo+bar+".to_string()),
                Edn::Symbol("+'-".to_string()),
                Edn::Symbol("'-+".to_string()),
            ]))
        );

        assert!(Edn::from_str("(-foo( ba").is_err());
    }

    #[test]
    fn weird_input() {
        assert_eq!(err_as_string("{:a]"), "EdnError { code: UnmatchedDelimiter(']'), line: Some(1), column: Some(4), ptr: Some(3) }");
    }

    #[test]
    fn special_chars() {
        let edn = "[\\space \\@ \\` \\tab \\return \\newline \\# \\% \\' \\g \\( \\* \\j \\+ \\, \\l \\- \\. \\/ \\0 \\2 \\r \\: \\; \\< \\\\ \\] \\} \\~ \\? \\_]";

        assert_eq!(
            Edn::from_str(edn).unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Char(' '),
                Edn::Char('@'),
                Edn::Char('`'),
                Edn::Char('\t'),
                Edn::Char('\r'),
                Edn::Char('\n'),
                Edn::Char('#'),
                Edn::Char('%'),
                Edn::Char('\''),
                Edn::Char('g'),
                Edn::Char('('),
                Edn::Char('*'),
                Edn::Char('j'),
                Edn::Char('+'),
                Edn::Char(','),
                Edn::Char('l'),
                Edn::Char('-'),
                Edn::Char('.'),
                Edn::Char('/'),
                Edn::Char('0'),
                Edn::Char('2'),
                Edn::Char('r'),
                Edn::Char(':'),
                Edn::Char(';'),
                Edn::Char('<'),
                Edn::Char('\\'),
                Edn::Char(']'),
                Edn::Char('}'),
                Edn::Char('~'),
                Edn::Char('?'),
                Edn::Char('_'),
            ]))
        );
    }

    #[test]
    fn invalid_edn() {
        assert!(Edn::from_str("{ :foo 42 :foo 43 }").is_err());
        assert!(Edn::from_str("{ :[0x42] 42 }").is_err());
        assert!(Edn::from_str("\\cats").is_err());
    }
}
