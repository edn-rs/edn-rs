use edn_rs::{map, Deserialize, Edn, EdnError, Map};

#[derive(Debug, PartialEq)]
struct Person {
    name: String,
    age: u64,
}

impl Deserialize for Person {
    fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
        Ok(Self {
            name: edn_rs::from_edn(&edn[":name"])?,
            age: edn_rs::from_edn(&edn[":age"])?,
        })
    }
}

fn person_ok() -> Result<(), EdnError> {
    let edn = Edn::Map(Map::new(map! {
        ":name".to_string() => Edn::Str("rose".to_string()),
        ":age".to_string() => Edn::UInt(66)
    }));
    let person: Person = edn_rs::from_edn(&edn)?;

    println!("{person:?}");
    // Person { name: "rose", age: 66 }

    assert_eq!(
        person,
        Person {
            name: "rose".to_string(),
            age: 66,
        }
    );
    Ok(())
}

fn person_mistyped() -> Result<(), EdnError> {
    let bad_edn = Edn::Map(Map::new(map! {
        ":name".to_string() => Edn::Str("rose".to_string()),
        ":age".to_string() => Edn::Str("some text".to_string())
    }));
    let person: Result<Person, EdnError> = edn_rs::from_edn(&bad_edn);

    assert!(person.is_err());

    Ok(())
}

fn main() -> Result<(), EdnError> {
    person_ok()?;
    person_mistyped()?;

    Ok(())
}

#[test]
fn test_person_ok() {
    let _ = person_ok().unwrap();
}

#[test]
fn test_person_mistyped() {
    let _ = person_mistyped();
}
