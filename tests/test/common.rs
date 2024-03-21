use core::str::FromStr;
use edn_rs::Edn;

pub fn err_as_string(s: &str) -> String {
    let err = Edn::from_str(s).err().unwrap();
    format!("{err:?}")
}
