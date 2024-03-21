#[cfg(feature = "json")]
mod tests {
    use edn_rs::json_to_edn;

    #[test]
    fn emits_helloworld_edn() {
        let json = String::from("{\"hello\": \"world\"}");
        let edn = String::from("{:hello \"world\"}");

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn emits_helloworld_from_str_edn() {
        let json = "{\"hello\": \"world\"}";
        let edn = "{:hello \"world\"}";

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn replaces_whitespace_in_keys_by_dash() {
        let json = String::from("{\"hello world\": \"julia\"}");
        let edn = String::from("{:hello-world \"julia\"}");

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn emits_nil_edn() {
        let json = String::from("{\"hello\": null}");
        let edn = String::from("{:hello nil}");

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn emits_char_edn() {
        let json = String::from("{\"hello\": 'c'}");
        let edn = String::from("{:hello \\c}");

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn emits_number_edn() {
        let json = String::from("{\"multi_string with underscore\": 545643}");
        let edn = String::from("{:multi-string-with-underscore 545643}");

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn multiline_json_to_edn() {
        let json = String::from(
            "{
            \"hello\": [
                {
                    \"country name\": \"brazil\",
                    \"word\": \"mundo\"
                },
                {
                    \"country name\": \"usa\",
                    \"word\": \"world\"
                }
            ]
        }",
        );
        let edn = String::from(
            "{
            :hello [
                {
                    :country-name \"brazil\",
                    :word \"mundo\"
                },
                {
                    :country-name \"usa\",
                    :word \"world\"
                }
            ]
        }",
        );

        assert_eq!(edn, json_to_edn(json));
    }

    #[test]
    fn regression_str_to_uint_test() {
        use edn_derive::Deserialize;
        #[derive(Deserialize, Debug, PartialEq)]
        struct A {
            amount: u64,
        }

        let a: A = edn_rs::from_str("{ :amount \"123\" }").unwrap();
        assert_eq!(a, A { amount: 123 });
    }

    #[test]
    fn to_json() {
        use edn_rs::edn::{Edn, List, Map, Set, Vector};
        use edn_rs::{map, set};

        let edn = Edn::Vector(Vector::new(vec![
            Edn::Int(1),
            Edn::Double(1.2.into()),
            Edn::UInt(3),
            Edn::List(List::new(vec![
                Edn::Bool(false),
                Edn::Key(":f".to_string()),
                Edn::Nil,
                Edn::Rational((3, 4)),
                Edn::Set(Set::new(set! {
                    Edn::Rational((3, 4))
                })),
            ])),
            Edn::Map(Map::new(map![
                    String::from("false") => Edn::Key(":f".to_string()),
                    String::from("nil") => Edn::Rational((3, 4)),
                    String::from(":my-crazy-map") => Edn::Map(Map::new(map![
                        String::from("false") => Edn::Map(
                            Map::new( map![
                                String::from(":f") => Edn::Key(String::from(":b"))
                            ])),
                        String::from("nil") => Edn::Vector(
                            Vector::new( vec![
                                Edn::Rational((3, 4)),
                                Edn::Int(1i64)
                            ]))
                ]))
            ])),
        ]));

        assert_eq!(edn.to_json(),
            "[1, 1.2, 3, [false, \"f\", null, 0.75, [0.75]], {\"myCrazyMap\": {\"false\": {\"f\": \"b\"}, \"nil\": [0.75, 1]}, \"false\": \"f\", \"nil\": 0.75}]");
    }
}
