#[cfg(feature = "sets")]
use crate::edn::Set;
use crate::edn::{Edn, Error, List, Map, Vector};
use std::collections::BTreeMap;
#[cfg(feature = "sets")]
use std::collections::BTreeSet;

const DELIMITERS: [char; 8] = [',', ']', '}', ')', ';', '(', '[', '{'];

pub fn tokenize(edn: &str) -> std::iter::Enumerate<std::str::Chars> {
    edn.chars().enumerate()
}

pub fn parse(
    c: Option<(usize, char)>,
    chars: &mut std::iter::Enumerate<std::str::Chars>,
) -> Result<Edn, Error> {
    (parse_internal(c, chars)?).map_or_else(|| Ok(Edn::Empty), Ok)
}

fn parse_internal(
    c: Option<(usize, char)>,
    chars: &mut std::iter::Enumerate<std::str::Chars>,
) -> Result<Option<Edn>, Error> {
    Ok(match c {
        Some((_, '[')) => Some(read_vec(chars)?),
        Some((_, '(')) => Some(read_list(chars)?),
        Some((_, '#')) => tagged_or_set_or_discard(chars)?,
        Some((_, '{')) => Some(read_map(chars)?),
        Some((_, ';')) => {
            // Consumes the content
            chars.find(|c| c.1 == '\n');
            read_if_not_container_end(chars)?
        }
        Some((_, s)) if s.is_whitespace() || s == ',' => read_if_not_container_end(chars)?,
        None => None,
        edn => Some(parse_edn(edn, chars)?),
    })
}

#[allow(clippy::module_name_repetitions)]
pub fn parse_edn(
    c: Option<(usize, char)>,
    chars: &mut std::iter::Enumerate<std::str::Chars>,
) -> Result<Edn, Error> {
    match c {
        Some((_, '\"')) => read_str(chars),
        Some((_, ':')) => read_key_or_nsmap(chars),
        Some((_, n)) if n.is_numeric() => Ok(read_number(n, chars)?),
        Some((_, n))
            if (n == '-' || n == '+')
                && chars
                    .clone()
                    .peekable()
                    .peek()
                    .is_some_and(|n| n.1.is_numeric()) =>
        {
            Ok(read_number(n, chars)?)
        }
        Some((_, '\\')) => Ok(read_char(chars)?),
        Some((_, b)) if b == 't' || b == 'f' || b == 'n' => Ok(read_bool_or_nil(b, chars)?),
        Some((_, a)) => Ok(read_symbol(a, chars)?),
        None => Err(Error::ParseEdn("Edn could not be parsed".to_string())),
    }
}

fn tagged_or_set_or_discard(
    chars: &mut std::iter::Enumerate<std::str::Chars>,
) -> Result<Option<Edn>, Error> {
    match chars.clone().next() {
        Some((_, '{')) => read_set(chars).map(Some),
        Some((_, '_')) => read_discard(chars),
        _ => read_tagged(chars).map(Some),
    }
}

fn read_key_or_nsmap(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let mut key_chars = chars.clone().take_while(|c| {
        !c.1.is_whitespace() && c.1 != ',' && c.1 != ')' && c.1 != ']' && c.1 != '}' && c.1 != ';'
    });
    let c_len = key_chars.clone().count();

    Ok(match key_chars.find(|c| c.1 == '{') {
        Some(_) => read_namespaced_map(chars)?,
        None => read_key(chars, c_len),
    })
}

fn read_key(chars: &mut std::iter::Enumerate<std::str::Chars>, c_len: usize) -> Edn {
    let mut key = String::from(":");
    let key_chars = chars.take(c_len).map(|c| c.1).collect::<String>();
    key.push_str(&key_chars);
    Edn::Key(key)
}

fn read_str(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let result = chars.try_fold(
        (false, String::new()),
        |(last_was_escape, mut s), (_, c)| {
            if last_was_escape {
                // Supported escape characters, per https://github.com/edn-format/edn#strings
                match c {
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    'n' => s.push('\n'),
                    '\\' => s.push('\\'),
                    '\"' => s.push('\"'),
                    _ => {
                        return Err(Err(Error::ParseEdn(format!(
                            "Invalid escape sequence \\{c}"
                        ))))
                    }
                };

                Ok((false, s))
            } else if c == '\"' {
                // Unescaped quote means we're done
                Err(Ok(s))
            } else if c == '\\' {
                Ok((true, s))
            } else {
                s.push(c);
                Ok((false, s))
            }
        },
    );

    match result {
        // An Ok means we actually finished parsing *without* seeing the end of the string, so that's
        // an error.
        Ok(_) => Err(Error::ParseEdn("Unterminated string".to_string())),
        Err(Err(e)) => Err(e),
        Err(Ok(string)) => Ok(Edn::Str(string)),
    }
}

fn read_symbol(a: char, chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let c_len = chars
        .clone()
        .enumerate()
        .take_while(|&(i, c)| i <= 200 && !c.1.is_whitespace() && !DELIMITERS.contains(&c.1))
        .count();
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;

    if a.is_whitespace() {
        return Err(Error::ParseEdn(format!(
            "\"{a}\" could not be parsed at char count {i}"
        )));
    }

    let mut symbol = String::from(a);
    let symbol_chars = chars.take(c_len).map(|c| c.1).collect::<String>();
    symbol.push_str(&symbol_chars);
    Ok(Edn::Symbol(symbol))
}

fn read_tagged(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let tag = chars
        .take_while(|c| !c.1.is_whitespace() && c.1 != ',')
        .map(|c| c.1)
        .collect::<String>();

    if tag.starts_with("inst") {
        return Ok(Edn::Inst(
            chars
                .skip_while(|c| c.1 == '\"' || c.1.is_whitespace())
                .take_while(|c| c.1 != '\"')
                .map(|c| c.1)
                .collect::<String>(),
        ));
    }

    if tag.starts_with("uuid") {
        return Ok(Edn::Uuid(
            chars
                .skip_while(|c| c.1 == '\"' || c.1.is_whitespace())
                .take_while(|c| c.1 != '\"')
                .map(|c| c.1)
                .collect::<String>(),
        ));
    }

    Ok(Edn::Tagged(tag, Box::new(parse(chars.next(), chars)?)))
}

fn read_discard(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Option<Edn>, Error> {
    let _discard_underscore = chars.next();
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    match parse(chars.next(), chars) {
        Err(e) => Err(e),
        Ok(Edn::Empty) => Err(Error::ParseEdn(format!(
            "Discard sequence must have a following element at char count {i}"
        ))),
        _ => read_if_not_container_end(chars),
    }
}

fn read_number(n: char, chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let c_len = chars
        .clone()
        .take_while(|(_, c)| !c.is_whitespace() && !DELIMITERS.contains(c))
        .count();
    let (number, radix) = {
        let mut number = String::new();
        // The EDN spec allows for a redundant '+' symbol, we just ignore it.
        if n != '+' {
            number.push(n);
        }
        for (_, c) in chars.take(c_len) {
            number.push(c);
        }
        if number.to_lowercase().starts_with("0x") {
            number.remove(0);
            number.remove(0);
            (number, 16)
        } else if number.to_lowercase().starts_with("-0x") {
            number.remove(1);
            number.remove(1);
            (number, 16)
        } else if let Some(index) = number.to_lowercase().find('r') {
            let negative = number.starts_with('-');
            let radix = {
                if negative {
                    (number[1..index]).parse::<u32>()
                } else {
                    (number[0..index]).parse::<u32>()
                }
            };
            match radix {
                Ok(r) => {
                    // from_str_radix panics if radix is not in the range from 2 to 36
                    if !(2..=36).contains(&r) {
                        return Err(Error::ParseEdn(format!("Radix of {r} is out of bounds")));
                    }

                    if negative {
                        for _ in 0..(index) {
                            number.remove(1);
                        }
                    } else {
                        for _ in 0..=index {
                            number.remove(0);
                        }
                    }
                    (number, r)
                }
                Err(e) => {
                    return Err(Error::ParseEdn(format!(
                        "{e} while trying to parse radix from {number}"
                    )));
                }
            }
        } else {
            (number, 10)
        }
    };

    match number {
        n if (n.contains('E') || n.contains('e')) && n.parse::<f64>().is_ok() => {
            Ok(Edn::Double(n.parse::<f64>()?.into()))
        }
        n if u64::from_str_radix(&n, radix).is_ok() => {
            Ok(Edn::UInt(u64::from_str_radix(&n, radix)?))
        }
        n if i64::from_str_radix(&n, radix).is_ok() => {
            Ok(Edn::Int(i64::from_str_radix(&n, radix)?))
        }
        n if n.parse::<f64>().is_ok() => Ok(Edn::Double(n.parse::<f64>()?.into())),
        n if n.contains('/') && n.split('/').all(|d| d.parse::<f64>().is_ok()) => {
            Ok(Edn::Rational(n))
        }
        n if n.to_uppercase().chars().filter(|c| c == &'E').count() > 1 => {
            let mut n = n.chars();
            read_symbol(n.next().unwrap_or(' '), &mut n.enumerate())
        }
        _ => Err(Error::ParseEdn(format!(
            "{number} could not be parsed at char count {i} with radix {radix}"
        ))),
    }
}

fn read_char(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let c = chars.next();
    c.ok_or(format!("{c:?} could not be parsed at char count {i}"))
        .map(|c| c.1)
        .map(Edn::Char)
        .map_err(Error::ParseEdn)
}

fn read_bool_or_nil(
    c: char,
    chars: &mut std::iter::Enumerate<std::str::Chars>,
) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    match c {
        't' if {
            let val = chars
                .clone()
                .take_while(|(_, c)| !c.is_whitespace() && !DELIMITERS.contains(c))
                .map(|c| c.1)
                .collect::<String>();
            val.eq("rue")
        } =>
        {
            let mut string = String::new();
            let t = chars.take(3).map(|c| c.1).collect::<String>();
            string.push(c);
            string.push_str(&t);
            Ok(Edn::Bool(string.parse::<bool>()?))
        }
        'f' if {
            let val = chars
                .clone()
                .take_while(|(_, c)| !c.is_whitespace() && !DELIMITERS.contains(c))
                .map(|c| c.1)
                .collect::<String>();
            val.eq("alse")
        } =>
        {
            let mut string = String::new();
            let f = chars.take(4).map(|c| c.1).collect::<String>();
            string.push(c);
            string.push_str(&f);
            Ok(Edn::Bool(string.parse::<bool>()?))
        }
        'n' if {
            let val = chars
                .clone()
                .take_while(|(_, c)| !c.is_whitespace() && !DELIMITERS.contains(c))
                .map(|c| c.1)
                .collect::<String>();
            val.eq("il")
        } =>
        {
            let mut string = String::new();
            let n = chars.take(2).map(|c| c.1).collect::<String>();
            string.push(c);
            string.push_str(&n);
            match &string[..] {
                "nil" => Ok(Edn::Nil),
                _ => Err(Error::ParseEdn(format!(
                    "{string} could not be parsed at char count {i}"
                ))),
            }
        }
        _ => read_symbol(c, chars),
    }
}

fn read_vec(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let mut res: Vec<Edn> = vec![];
    loop {
        match chars.next() {
            Some((_, ']')) => return Ok(Edn::Vector(Vector::new(res))),
            Some(c) => {
                if let Some(e) = parse_internal(Some(c), chars)? {
                    res.push(e);
                }
            }
            err => {
                return Err(Error::ParseEdn(format!(
                    "{err:?} could not be parsed at char count {i}"
                )))
            }
        }
    }
}

fn read_list(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let mut res: Vec<Edn> = vec![];
    loop {
        match chars.next() {
            Some((_, ')')) => return Ok(Edn::List(List::new(res))),
            Some(c) => {
                if let Some(e) = parse_internal(Some(c), chars)? {
                    res.push(e);
                }
            }
            err => {
                return Err(Error::ParseEdn(format!(
                    "{err:?} could not be parsed at char count {i}"
                )))
            }
        }
    }
}

#[cfg(feature = "sets")]
fn read_set(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let _discard_brackets = chars.next();
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let mut res: BTreeSet<Edn> = BTreeSet::new();
    loop {
        match chars.next() {
            Some((_, '}')) => return Ok(Edn::Set(Set::new(res))),
            Some(c) => {
                if let Some(e) = parse_internal(Some(c), chars)? {
                    res.insert(e);
                }
            }
            err => {
                return Err(Error::ParseEdn(format!(
                    "{err:?} could not be parsed at char count {i}"
                )))
            }
        }
    }
}

#[cfg(not(feature = "sets"))]
fn read_set(_chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    Err(Error::ParseEdn(
        "Could not parse set due to feature not being enabled".to_string(),
    ))
}

fn read_namespaced_map(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let mut res: BTreeMap<String, Edn> = BTreeMap::new();
    let mut key: Option<Edn> = None;
    let mut val: Option<Edn> = None;
    let namespace = chars
        .take_while(|c| c.1 != '{')
        .map(|c| c.1)
        .collect::<String>();

    loop {
        match chars.next() {
            Some((_, '}')) => return Ok(Edn::NamespacedMap(namespace, Map::new(res))),
            Some(c) => {
                if key.is_some() {
                    val = Some(parse(Some(c), chars)?);
                } else {
                    key = parse_internal(Some(c), chars)?;
                }
            }
            err => {
                return Err(Error::ParseEdn(format!(
                    "{err:?} could not be parsed at char count {i}"
                )))
            }
        }

        if key.is_some() && val.is_some() {
            res.insert(key.unwrap().to_string(), val.unwrap());
            key = None;
            val = None;
        }
    }
}

fn read_map(chars: &mut std::iter::Enumerate<std::str::Chars>) -> Result<Edn, Error> {
    let i = chars
        .clone()
        .next()
        .ok_or_else(|| Error::ParseEdn("Could not identify symbol index".to_string()))?
        .0;
    let mut res: BTreeMap<String, Edn> = BTreeMap::new();
    let mut key: Option<Edn> = None;
    let mut val: Option<Edn> = None;
    loop {
        match chars.next() {
            Some((_, '}')) => return Ok(Edn::Map(Map::new(res))),
            Some(c) => {
                if key.is_some() {
                    val = Some(parse(Some(c), chars)?);
                } else {
                    key = parse_internal(Some(c), chars)?;
                }
            }
            err => {
                return Err(Error::ParseEdn(format!(
                    "{err:?} could not be parsed at char count {i}"
                )))
            }
        }

        if key.is_some() && val.is_some() {
            res.insert(key.unwrap().to_string(), val.unwrap());
            key = None;
            val = None;
        }
    }
}

fn read_if_not_container_end(
    chars: &mut std::iter::Enumerate<std::str::Chars>,
) -> Result<Option<Edn>, Error> {
    Ok(match chars.clone().next() {
        Some(c) if c.1 == ']' || c.1 == ')' || c.1 == '}' => None,
        Some(_) => parse_internal(chars.next(), chars)?,
        None => None,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::edn::Map;
    #[cfg(feature = "sets")]
    use crate::edn::Set;
    use crate::{map, set};

    #[test]
    fn parse_empty() {
        let mut edn = "".chars().enumerate();

        assert_eq!(parse(edn.next(), &mut edn).unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_whitespace_only() {
        let mut edn = "
                          \r\n"
            .chars()
            .enumerate();

        assert_eq!(parse(edn.next(), &mut edn).unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_commas_are_whitespace() {
        let mut edn = ",,,,, \r\n,,,".chars().enumerate();

        assert_eq!(parse(edn.next(), &mut edn).unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_keyword() {
        let mut key = ":keyword".chars().enumerate();

        assert_eq!(
            parse_edn(key.next(), &mut key).unwrap(),
            Edn::Key(":keyword".to_string())
        )
    }

    #[test]
    fn parse_str() {
        let mut string = "\"hello world, from      RUST\"".chars().enumerate();

        assert_eq!(
            parse_edn(string.next(), &mut string).unwrap(),
            Edn::Str("hello world, from      RUST".to_string())
        )
    }

    #[test]
    fn parse_str_top_level_comment() {
        let mut string = ";;; hello world string example\n\n;; deserialize the following string\n\n\"hello world, from      RUST\"".chars().enumerate();

        assert_eq!(
            parse(string.next(), &mut string).unwrap(),
            Edn::Str("hello world, from      RUST".to_string())
        )
    }

    #[test]
    fn parse_str_top_level_comment_whitespace() {
        let mut string = "\n;;; hello world string example\n\n;; deserialize the following string\n\n,,\"hello world, from      RUST\"".chars().enumerate();

        assert_eq!(
            parse(string.next(), &mut string).unwrap(),
            Edn::Str("hello world, from      RUST".to_string())
        )
    }

    #[test]
    fn parse_str_looks_like_comment() {
        let mut string = "\";;; hello world, from      RUST\n\"".chars().enumerate();

        assert_eq!(
            parse_edn(string.next(), &mut string).unwrap(),
            Edn::Str(";;; hello world, from      RUST\n".to_string())
        )
    }

    #[test]
    fn parse_str_with_escaped_characters() {
        let mut string = r##""hello\n \r \t \"world\" with escaped \\ characters""##
            .chars()
            .enumerate();

        assert_eq!(
            parse_edn(string.next(), &mut string).unwrap(),
            Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())
        )
    }

    #[test]
    fn parse_str_with_invalid_escape() {
        let mut string = r##""hello\n \r \t \"world\" with escaped \\ \g characters""##
            .chars()
            .enumerate();

        assert_eq!(
            parse_edn(string.next(), &mut string),
            Err(Error::ParseEdn("Invalid escape sequence \\g".to_string()))
        )
    }

    #[test]
    fn parse_unterminated_string() {
        let mut string = r##""hello\n \r \t \"world\" with escaped \\ characters"##
            .chars()
            .enumerate();

        assert_eq!(
            parse_edn(string.next(), &mut string),
            Err(Error::ParseEdn("Unterminated string".to_string()))
        )
    }

    #[test]
    fn parse_number() {
        use crate::edn;
        let mut uint = "143".chars().enumerate();
        let mut int = "-435143".chars().enumerate();
        let mut f = "-43.5143".chars().enumerate();
        let mut r = "43/5143".chars().enumerate();
        let mut big_f64 = "999999999999999999999.0".chars().enumerate();
        assert_eq!(parse_edn(uint.next(), &mut uint).unwrap(), Edn::UInt(143));
        assert_eq!(parse_edn(int.next(), &mut int).unwrap(), Edn::Int(-435143));
        assert_eq!(
            parse_edn(f.next(), &mut f).unwrap(),
            Edn::Double(edn::Double::from(-43.5143))
        );
        assert_eq!(
            parse_edn(r.next(), &mut r).unwrap(),
            Edn::Rational("43/5143".to_string())
        );
        assert_eq!(
            parse_edn(big_f64.next(), &mut big_f64).unwrap(),
            Edn::Double(edn::Double::from(1e21f64))
        );
    }

    #[test]
    fn parse_char() {
        let mut c = "\\k".chars().enumerate();

        assert_eq!(parse_edn(c.next(), &mut c).unwrap(), Edn::Char('k'))
    }

    #[test]
    fn parse_bool_or_nil() {
        let mut t = "true".chars().enumerate();
        let mut f = "false".chars().enumerate();
        let mut n = "nil".chars().enumerate();
        let mut s = "\"true\"".chars().enumerate();
        assert_eq!(parse_edn(t.next(), &mut t).unwrap(), Edn::Bool(true));
        assert_eq!(parse_edn(f.next(), &mut f).unwrap(), Edn::Bool(false));
        assert_eq!(parse_edn(n.next(), &mut n).unwrap(), Edn::Nil);
        assert_eq!(
            parse_edn(s.next(), &mut s).unwrap(),
            Edn::Str("true".to_string())
        );
    }

    #[test]
    fn parse_simple_vec() {
        let mut edn = "[11 \"2\" 3.3 :b true \\c]".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        let mut edn = "[11 \"2\" 3.3 ; float in simple vec\n:b true \\c]"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        let mut edn = "[11 \"2\" 3.3 :b true \\c; char in simple vec\n]"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        let mut edn = "[
                         11 
                        \"2\" 
                         3.3 
                         ;; float in simple vec
                         :b 
                         true 
                         \\c
                       ]"
        .chars()
        .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        let mut edn = "[\ntrue\nfalse\nnil\n]".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Bool(true),
                Edn::Bool(false),
                Edn::Nil,
            ]))
        );
    }

    #[test]
    fn parse_bool_in_tab_simple_vec_str_literal() {
        let mut edn = "[\ttrue\tnil\tfalse\t]".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Bool(true),
                Edn::Nil,
                Edn::Bool(false),
            ]))
        );
    }

    #[test]
    fn parse_bool_in_crlf_newline_simple_vec_str_literal() {
        let mut edn = "[\r\nnil\r\nfalse\r\ntrue\r\n]".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Vector(Vector::new(vec![
                Edn::Nil,
                Edn::Bool(false),
                Edn::Bool(true),
            ]))
        );
    }

    #[test]
    fn parse_list() {
        let mut edn = "(1 \"2\" 3.3 :b )".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
            ]))
        );
    }

    #[test]
    fn parse_comment_in_list() {
        let mut edn = "(1 \"2\"; string in list\n3.3 :b )".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        let mut edn = "(1 \"2\" 3.3 :b; keyword in list\n)".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::List(List::new(vec![
                Edn::UInt(1),
                Edn::Str("2".to_string()),
                Edn::Double(3.3.into()),
                Edn::Key(":b".to_string()),
            ]))
        );
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_set() {
        let mut edn = "#{true \\c 3 }".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Set(Set::new(set![
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3)
            ]))
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_set_with_commas() {
        let mut edn = "#{true, \\c, 3,four, }".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Set(Set::new(set![
                Edn::Symbol("four".to_string()),
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3),
            ]))
        )
    }

    #[test]
    #[cfg(not(feature = "sets"))]
    fn parse_set_without_set_feature() {
        let mut edn = "#{true, \\c, 3,four, }".chars().enumerate();
        let res = parse(edn.next(), &mut edn);

        assert_eq!(
            res,
            Err(Error::ParseEdn(
                "Could not parse set due to feature not being enabled".to_string()
            ))
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_comment_in_set() {
        let mut edn = "#{true ; bool true in a set\n \\c 3 }".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Set(Set::new(set![
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3)
            ]))
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_true_false_nil_with_comments_in_set() {
        let mut edn = "#{true;this is true\nfalse;this is false\nnil;this is nil\n}"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Set(Set::new(set![Edn::Bool(true), Edn::Bool(false), Edn::Nil,]))
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_comment_in_set_end() {
        let mut edn = "#{true \\c 3; int 3 in a set\n}".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Set(Set::new(set![
                Edn::Bool(true),
                Edn::Char('c'),
                Edn::UInt(3)
            ]))
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_complex() {
        let mut edn = "[:b ( 5 \\c #{true \\c 3 } ) ]".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_comment_complex() {
        let mut edn = "[:b ( 5 \\c #{true \\c; char c in a set\n3 } ) ]"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
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
        )
    }

    #[test]
    fn parse_simple_map() {
        let mut edn = "{:a \"2\" :b false :c nil }".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("2".to_string()),
                ":b".to_string() => Edn::Bool(false), ":c".to_string() => Edn::Nil}
            ))
        );
    }

    #[test]
    fn parse_inst() {
        let mut edn = "{:date  #inst \"2020-07-16T21:53:14.628-00:00\"}"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(map! {
                ":date".to_string() => Edn::Inst("2020-07-16T21:53:14.628-00:00".to_string())
            }))
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_edn_with_inst() {
        let mut edn =
            "#{ :a :b {:c :d :date  #inst \"2020-07-16T21:53:14.628-00:00\" ::c ::d} nil}"
                .chars()
                .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Set(Set::new(set! {
                Edn::Key(":a".to_string()),
                Edn::Key(":b".to_string()),
                Edn::Map(Map::new(map! {
                    ":c".to_string() => Edn::Key(":d".to_string()),
                    ":date".to_string() => Edn::Inst("2020-07-16T21:53:14.628-00:00".to_string()),
                    "::c".to_string() => Edn::Key("::d".to_string())
                })),
                Edn::Nil
            }))
        )
    }

    #[test]
    fn parse_tagged_int() {
        let mut edn = "#iasdf 234".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(String::from("iasdf"), Box::new(Edn::UInt(234)))
        )
    }

    #[test]
    fn parse_discard_valid() {
        let mut edn = "#_iasdf 234".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(res, Edn::UInt(234))
    }

    #[test]
    fn parse_discard_invalid() {
        let mut edn = "#_{ 234".chars().enumerate();
        let res = parse(edn.next(), &mut edn);

        assert_eq!(
            res,
            Err(Error::ParseEdn(
                "None could not be parsed at char count 3".to_string()
            ))
        )
    }

    #[test]
    fn parse_discard_space_valid() {
        let mut edn = "#_ ,, 234 567".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(res, Edn::UInt(567))
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_discard_space_invalid() {
        let mut edn = "#_ ,, #{hello, this will be discarded} #_{so will this} #{this is invalid"
            .chars()
            .enumerate();
        let res = parse(edn.next(), &mut edn);

        assert_eq!(
            res,
            Err(Error::ParseEdn(
                "None could not be parsed at char count 58".to_string()
            ))
        )
    }

    #[test]
    fn parse_discard_empty() {
        let mut edn = "#_ ,, foo".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(res, Edn::Empty)
    }

    #[test]
    fn parse_discard_repeat_empty() {
        let mut edn = "#_ ,, #_{discard again} #_ {:and :again} :okay"
            .chars()
            .enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(res, Edn::Empty)
    }

    #[test]
    fn parse_discard_repeat_not_empty() {
        let mut edn = "#_ ,, #_{discard again} #_ {:and :again} :okay {:a map}"
            .chars()
            .enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Symbol("map".to_string())}
            ))
        )
    }

    #[test]
    fn parse_discard_no_follow_element() {
        let mut edn = "#_ ,, ".chars().enumerate();
        let res = parse(edn.next(), &mut edn);

        assert_eq!(
            res,
            Err(Error::ParseEdn(
                "Discard sequence must have a following element at char count 2".to_string()
            ))
        )
    }

    #[test]
    fn parse_discard_end_of_seq() {
        let mut edn = "[:foo #_ foo]".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Vector(Vector::new(vec![Edn::Key(":foo".to_string())]))
        );
    }

    #[test]
    fn parse_discard_end_of_seq_no_follow() {
        let mut edn = "[:foo #_ ]".chars().enumerate();
        let res = parse(edn.next(), &mut edn);

        assert_eq!(
            res,
            Err(Error::ParseEdn(
                "Discard sequence must have a following element at char count 8".to_string()
            ))
        )
    }

    #[test]
    fn parse_discard_inside_seq() {
        let mut edn = "#_\"random comment\" [:a :b :c #_(:hello :world) :d]"
            .chars()
            .enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();
        assert_eq!(
            res,
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
        let mut edn = "{ :a :something, :b false, :c nil, }".chars().enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Key(":something".to_string()),
                ":b".to_string() => Edn::Bool(false), ":c".to_string() => Edn::Nil}
            ))
        );
    }

    #[test]
    fn parse_map_with_special_char_str1() {
        let mut edn = "{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_only() {
        let mut edn = " ;;; this is a comment\n".chars().enumerate();

        assert_eq!(parse(edn.next(), &mut edn).unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_comment_only_no_newline() {
        let mut edn = " ;;; this is a comment".chars().enumerate();

        assert_eq!(parse(edn.next(), &mut edn).unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_comment_multiple() {
        let mut edn = " ;;; comment 1\n ;;; comment 2\n ;;; comment 3\n\n"
            .chars()
            .enumerate();

        assert_eq!(parse(edn.next(), &mut edn).unwrap(), Edn::Empty);
    }

    #[test]
    fn parse_comment_top_level() {
        let mut edn = " ;; this is a map\n{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_inside_map() {
        let mut edn =
            "{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" ; escaped chars\n }"
                .chars()
                .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_end_of_file() {
        let mut edn = ";; this is a map\n{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }\n ;; end of file\n"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_comment_end_of_file_no_newline() {
        let mut edn = ";; this is a map\n{ :a \"hello\n \r \t \\\"world\\\" with escaped \\\\ characters\" }\n ;; end of file"
            .chars()
            .enumerate();

        assert_eq!(
            parse(edn.next(), &mut edn).unwrap(),
            Edn::Map(Map::new(
                map! {":a".to_string() => Edn::Str("hello\n \r \t \"world\" with escaped \\ characters".to_string())}
            ))
        );
    }

    #[test]
    fn parse_tagged_vec() {
        let mut edn = "#domain/model [1 2 3]".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Vector(Vector::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        )
    }

    #[test]
    fn parse_tagged_vec_with_comment() {
        let mut edn = "#domain/model ; tagging this vector\n [1 2 3]"
            .chars()
            .enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Vector(Vector::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        )
    }

    #[test]
    fn parse_map_with_tagged_vec() {
        let mut edn = "{ :model #domain/model [1 2 3] :int 2 }"
            .chars()
            .enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
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
        )
    }

    #[test]
    fn parse_tagged_list() {
        let mut edn = "#domain/model (1 2 3)".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::List(List::new(vec![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        )
    }

    #[test]
    fn parse_tagged_str() {
        let mut edn = "#domain/model \"hello\"".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Str(String::from("hello")))
            )
        )
    }

    #[test]
    #[cfg(feature = "sets")]
    fn parse_tagged_set() {
        let mut edn = "#domain/model #{1 2 3}".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Set(Set::new(set![
                    Edn::UInt(1),
                    Edn::UInt(2),
                    Edn::UInt(3)
                ])))
            )
        )
    }

    #[test]
    fn parse_tagged_map() {
        let mut edn = "#domain/model {1 2 3 4}".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        assert_eq!(
            res,
            Edn::Tagged(
                String::from("domain/model"),
                Box::new(Edn::Map(Map::new(map! {
                    "1".to_string() =>
                    Edn::UInt(2),
                    "3".to_string() =>
                    Edn::UInt(4)
                })))
            )
        )
    }

    #[test]
    fn parse_tagged_map_anything() {
        let mut edn = "#domain/model \n;; cool a tagged map!!!\n {1 \"hello\" 3 [[1 2] [2 3] [3,, 4]] #keyword, :4,,, {:cool-tagged #yay ;; what a tag inside a tagged map?!\n {:stuff \"hehe\"}}, 5 #wow {:a, :b}}".chars().enumerate();
        let res = parse(edn.next(), &mut edn).unwrap();

        println!("{:#?}\n\n", res);

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
        )
    }

    #[test]
    fn parse_exp() {
        let mut edn = "5.01122771367421E15".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::Double(5011227713674210f64.into()))
        );
    }

    #[test]
    fn parse_numberic_symbol_with_doube_e() {
        let mut edn = "5011227E71367421E12".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::Symbol("5011227E71367421E12".to_string()))
        );
    }

    #[test]
    fn parse_exp_plus_sign() {
        let mut edn = "5.01122771367421E+12".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::Double(5011227713674.210f64.into()))
        );
    }

    #[test]
    fn parse_float_e_minus_12() {
        let mut edn = "0.00000000000501122771367421".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::Double(5.01122771367421e-12.into()))
        );
    }

    #[test]
    fn parse_exp_minus_sign() {
        let mut edn = "5.01122771367421e-12".chars().enumerate();
        let res = parse(edn.next(), &mut edn);

        assert_eq!(res, Ok(Edn::Double(0.00000000000501122771367421.into())));
        assert_eq!(res.unwrap().to_string(), "0.00000000000501122771367421");
    }

    #[test]
    fn parse_0x_ints() {
        let mut edn = "0x2a".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::UInt(42)));

        let mut edn = "-0X2A".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::Int(-42)));
    }

    #[test]
    fn parse_radix_ints() {
        let mut edn = "16r2a".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::UInt(42)));

        let mut edn = "8r63".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::UInt(51)));

        let mut edn = "36rabcxyz".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::UInt(623741435)));

        let mut edn = "-16r2a".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::Int(-42)));

        let mut edn = "-32rFOObar".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::Int(-529280347)));
    }

    #[test]
    fn parse_invalid_ints() {
        let mut edn = "42invalid123".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Err(Error::ParseEdn(
                "42invalid123 could not be parsed at char count 1 with radix 10".to_string()
            ))
        );

        let mut edn = "0xxyz123".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Err(Error::ParseEdn(
                "xyz123 could not be parsed at char count 1 with radix 16".to_string()
            ))
        );

        let mut edn = "42rabcxzy".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Err(Error::ParseEdn("Radix of 42 is out of bounds".to_string()))
        );

        let mut edn = "42crazyrabcxzy".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Err(Error::ParseEdn(
                "invalid digit found in string while trying to parse radix from 42crazyrabcxzy"
                    .to_string()
            ))
        );
    }

    #[test]
    fn leading_plus_symbol_int() {
        let mut edn = "+42".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::UInt(42)));

        let mut edn = "+0x2a".chars().enumerate();
        assert_eq!(parse(edn.next(), &mut edn), Ok(Edn::UInt(42)));
    }

    #[test]
    fn lisp_quoted() {
        let mut edn = "('(symbol))".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::List(List::new(vec![
                Edn::Symbol("'".to_string()),
                Edn::List(List::new(vec![Edn::Symbol("symbol".to_string()),]))
            ])))
        );

        let mut edn = "(apply + '(1 2 3))".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::List(List::new(vec![
                Edn::Symbol("apply".to_string()),
                Edn::Symbol("+".to_string()),
                Edn::Symbol("'".to_string()),
                Edn::List(List::new(vec![Edn::UInt(1), Edn::UInt(2), Edn::UInt(3),]))
            ])))
        );

        let mut edn = "('(''symbol'foo''bar''))".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::List(List::new(vec![
                Edn::Symbol("'".to_string()),
                Edn::List(List::new(vec![Edn::Symbol(
                    "''symbol'foo''bar''".to_string()
                ),]))
            ])))
        );
    }

    #[test]
    fn minus_char_symbol() {
        let mut edn = "-foobar".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::Symbol("-foobar".to_string()))
        );

        let mut edn = "(+foobar +foo+bar+ +'- '-+)".chars().enumerate();
        assert_eq!(
            parse(edn.next(), &mut edn),
            Ok(Edn::List(List::new(vec![
                Edn::Symbol("+foobar".to_string()),
                Edn::Symbol("+foo+bar+".to_string()),
                Edn::Symbol("+'-".to_string()),
                Edn::Symbol("'-+".to_string()),
            ])))
        );

        let mut edn = "(-foo( ba".chars().enumerate();
        assert!(parse(edn.next(), &mut edn).is_err());
    }
}
