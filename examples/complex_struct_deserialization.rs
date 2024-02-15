use edn_rs::{Deserialize, Edn, EdnError};

#[derive(Debug, PartialEq)]
struct Another {
    name: String,
    age: u64,
    cool: bool,
}

impl Deserialize for Another {
    fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
        Ok(Self {
            name: edn_rs::from_edn(&edn[":name"])?,
            age: edn_rs::from_edn(&edn[":age"])?,
            cool: edn_rs::from_edn(&edn[":cool"])?,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Complex {
    list: Vec<Another>,
    nothing: (),
}

impl Deserialize for Complex {
    fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
        Ok(Self {
            list: edn_rs::from_edn(&edn[":list"])?,
            nothing: edn_rs::from_edn(&edn[":nothing"])?,
        })
    }
}

fn complex_ok() -> Result<(), EdnError> {
    let edn_str = "{ :list [{:name \"rose\" :age 66 :cool true}, {:name \"josh\" :age 33 :cool false}, {:name \"eva\" :age 296 :cool true}] :nothing nil }";
    let complex: Complex = edn_rs::from_str(edn_str)?;

    println!("{complex:?}");
    // Complex { list: [Another { name: "rose", age: 66, cool: true }, Another { name: "josh", age: 33, cool: false }, Another { name: "eva", age: 296, cool: true }] }

    assert_eq!(
        complex,
        Complex {
            list: vec![
                Another {
                    name: "rose".to_string(),
                    age: 66,
                    cool: true,
                },
                Another {
                    name: "josh".to_string(),
                    age: 33,
                    cool: false,
                },
                Another {
                    name: "eva".to_string(),
                    age: 296,
                    cool: true,
                },
            ],
            nothing: (),
        }
    );

    Ok(())
}

fn complex_wrong() -> Result<(), EdnError> {
    let bad_edn_str = "{:list [{:name \"rose\" :age \"some text\" :cool true}, {:name \"josh\" :age 33 :cool false}, {:name \"eva\" :age 296 :cool true}]}";
    let complex: Result<Complex, EdnError> = edn_rs::from_str(bad_edn_str);

    assert!(complex.is_err());

    Ok(())
}

fn main() -> Result<(), EdnError> {
    complex_ok()?;
    complex_wrong()?;
    Ok(())
}

#[test]
fn test_complex_ok() {
    let _ = complex_ok().unwrap();
}

#[test]
fn test_complex_wrong() {
    let _ = complex_wrong();
}
