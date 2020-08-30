use edn_rs::{Deserialize, Edn, EdnError};

#[derive(Debug, PartialEq)]
struct Another {
    name: String,
    age: usize,
    cool: bool,
}

impl Deserialize for Another {
    fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
        Ok(Self {
            name: Deserialize::deserialize(&edn[":name"])?,
            age: Deserialize::deserialize(&edn[":age"])?,
            cool: Deserialize::deserialize(&edn[":cool"])?,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Complex {
    id: usize,
    maybe: Option<Another>,
}

impl Deserialize for Complex {
    fn deserialize(edn: &Edn) -> Result<Self, EdnError> {
        Ok(Self {
            id: Deserialize::deserialize(&edn[":id"])?,
            maybe: Deserialize::deserialize(&edn[":maybe"])?,
        })
    }
}

fn main() -> Result<(), EdnError> {
    let edn_str = "{ :id 22 :maybe {:name \"rose\" :age 66 :cool true} }";
    let complex: Complex = edn_rs::from_str(edn_str)?;

    assert_eq!(
        complex,
        Complex {
            id: 22,
            maybe: Some(Another {
                name: "rose".to_string(),
                age: 66,
                cool: true,
            }),
        }
    );

    println!("{:?}", complex);
    // Complex { id: 22, maybe: Another { name: "rose", age: 66, cool: true } }

    let edn_str = "{ :id 1 }";
    let complex: Complex = edn_rs::from_str(edn_str)?;

    assert_eq!(complex, Complex { id: 1, maybe: None });

    println!("{:?}", complex);
    // Complex { id: 1, maybe: None }

    Ok(())
}