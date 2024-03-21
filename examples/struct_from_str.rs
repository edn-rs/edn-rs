use edn_rs::{Deserialize, Edn, EdnError};

#[derive(Debug, PartialEq)]
struct Person {
    name: String,
    age: u8,
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
    let edn_str = "  {:name \"rose\" :age 66  }  ";
    let person: Person = edn_rs::from_str(edn_str)?;

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
    let bad_edn_str = "{:name \"rose\" :age \"some text\" }";
    let person: Result<Person, EdnError> = edn_rs::from_str(bad_edn_str);

    assert!(person.is_err());
    Ok(())
}

fn person_overflow() -> Result<(), EdnError> {
    let overflow_edn_str = "  {:name \"rose\" :age 9002  }  ";
    let person: Result<Person, EdnError> = edn_rs::from_str(overflow_edn_str);

    assert!(person.is_err());
    Ok(())
}

fn main() -> Result<(), EdnError> {
    person_ok()?;
    person_mistyped()?;
    person_overflow()?;

    Ok(())
}

#[test]
fn test_person_ok() {
    let _ = person_ok().unwrap();
}

#[test]
fn test_person_mistyped() {
    let _ = person_mistyped().unwrap();
}

#[test]
fn test_person_overflow() {
    let _ = person_overflow().unwrap();
}
