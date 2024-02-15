#![allow(clippy::inline_always)]

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
#[cfg(feature = "sets")]
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::primitive::str;

use crate::edn::error::{Code, Error};
#[cfg(feature = "sets")]
use crate::edn::Set;
use crate::edn::{Edn, List, Map, Vector};

const DELIMITERS: [char; 8] = [',', ']', '}', ')', ';', '(', '[', '{'];

struct Walker<'w> {
    slice: &'w str,
    ptr: usize,
    column: usize,
    line: usize,
}

impl Walker<'_> {
    // Slurps until whitespace or delimiter, returning the slice.
    #[inline(always)]
    fn slurp_literal(&mut self) -> &str {
        let token = self.slice[self.ptr..]
            .split(|c: char| c.is_whitespace() || DELIMITERS.contains(&c))
            .next()
            .unwrap(); // At least an empty slice will always be on the first split, even on an empty str

        self.ptr += token.len();
        self.column += token.len();
        token
    }

    // Slurps a char. Special handling for chars that happen to be delimiters
    #[inline(always)]
    fn slurp_char(&mut self) -> &str {
        let starting_ptr = self.ptr;

        let mut ptr = 0;
        while let Some(c) = self.peek_next() {
            // first is always \\, second is always a char we want.
            // Handles edge cases of having a valid "\\[" but also "\\c[lolthisisvalidedn"
            if ptr > 1 && (c.is_whitespace() || DELIMITERS.contains(&c)) {
                break;
            }

            let _ = self.nibble_next();
            ptr += 1;
        }
        &self.slice[starting_ptr..starting_ptr + ptr]
    }

    // Slurps until whitespace or delimiter, returning the slice.
    #[inline(always)]
    fn slurp_tag(&mut self) -> &str {
        let token = self.slice[self.ptr..]
            .split(|c: char| c.is_whitespace() && c != ',')
            .next()
            .unwrap(); // At least an empty slice will always be on the first split, even on an empty str

        self.ptr += token.len();
        self.column += token.len();

        if token.ends_with(',') {
            return &token[0..token.len() - 1];
        }
        token
    }

    #[inline(always)]
    fn slurp_str(&mut self) -> Result<Edn, Error> {
        let _ = self.nibble_next(); // Consume the leading '"' char
        let mut s = String::new();
        let mut escape = false;
        loop {
            if let Some(c) = self.nibble_next() {
                if escape {
                    match c {
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        'n' => s.push('\n'),
                        '\\' => s.push('\\'),
                        '\"' => s.push('\"'),
                        _ => {
                            return Err(Error {
                                code: Code::InvalidEscape,
                                column: Some(self.column),
                                line: Some(self.line),
                                ptr: Some(self.ptr),
                            })
                        }
                    }
                    escape = false;
                } else if c == '\"' {
                    return Ok(Edn::Str(s));
                } else if c == '\\' {
                    escape = true;
                } else {
                    escape = false;
                    s.push(c);
                }
            } else {
                return Err(Error {
                    code: Code::UnexpectedEOF,
                    column: Some(self.column),
                    line: Some(self.line),
                    ptr: Some(self.ptr),
                });
            }
        }
    }

    // Nibbles away until the next new line
    #[inline(always)]
    fn nibble_newline(&mut self) {
        let len = self.slice[self.ptr..].split('\n').next().unwrap(); // At least an empty slice will always be on the first split, even on an empty str
        self.ptr += len.len();
        self.nibble_whitespace();
    }

    // Nibbles away until the start of the next form
    #[inline(always)]
    fn nibble_whitespace(&mut self) {
        while let Some(n) = self.peek_next() {
            if n == ',' || n.is_whitespace() {
                let _ = self.nibble_next();
                continue;
            }
            break;
        }
    }

    // Consumes next
    #[inline(always)]
    fn nibble_next(&mut self) -> Option<char> {
        let char = self.slice[self.ptr..].chars().next();
        if let Some(c) = char {
            self.ptr += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        char
    }

    // Peek into the next char
    #[inline(always)]
    fn peek_next(&mut self) -> Option<char> {
        self.slice[self.ptr..].chars().next()
    }
}

pub fn parse(edn: &str) -> Result<Edn, Error> {
    let mut walker = Walker {
        slice: edn,
        ptr: 0,
        column: 1,
        line: 1,
    };

    parse_internal(&mut walker)
}

#[inline]
fn parse_internal(walker: &mut Walker<'_>) -> Result<Edn, Error> {
    walker.nibble_whitespace();
    while let Some(next) = walker.peek_next() {
        let column_start = walker.column;
        let ptr_start = walker.ptr;
        let line_start = walker.line;
        if let Some(ret) = match next {
            '\\' => match parse_char(walker.slurp_char()) {
                Ok(edn) => Some(Ok(edn)),
                Err(code) => {
                    return Err(Error {
                        code,
                        line: Some(walker.line),
                        column: Some(column_start),
                        ptr: Some(walker.ptr),
                    })
                }
            },
            '\"' => Some(walker.slurp_str()),
            // comment. consume until a new line.
            ';' => {
                walker.nibble_newline();
                None
            }
            '[' => return parse_vector(walker),
            '(' => return parse_list(walker),
            '{' => return parse_map(walker),
            '#' => parse_tag_set_discard(walker)?.map(Ok),
            // non-string literal case
            _ => match edn_literal(walker.slurp_literal()) {
                Ok(edn) => Some(Ok(edn)),
                Err(code) => {
                    return Err(Error {
                        code,
                        line: Some(line_start),
                        column: Some(column_start),
                        ptr: Some(ptr_start),
                    })
                }
            },
        } {
            return ret;
        }
    }
    Ok(Edn::Empty)
}

#[inline]
fn parse_tag_set_discard(walker: &mut Walker<'_>) -> Result<Option<Edn>, Error> {
    let _ = walker.nibble_next(); // Consume the leading '#' char

    match walker.peek_next() {
        Some('{') => parse_set(walker).map(Some),
        Some('_') => parse_discard(walker),
        _ => parse_tag(walker).map(Some),
    }
}

#[inline]
fn parse_discard(walker: &mut Walker<'_>) -> Result<Option<Edn>, Error> {
    let _ = walker.nibble_next(); // Consume the leading '_' char
    Ok(match parse_internal(walker)? {
        Edn::Empty => {
            return Err(Error {
                code: Code::UnexpectedEOF,
                line: Some(walker.line),
                column: Some(walker.column),
                ptr: Some(walker.ptr),
            })
        }
        _ => match walker.peek_next() {
            Some(_) => Some(parse_internal(walker)?),
            None => None,
        },
    })
}

#[inline]
#[cfg(feature = "sets")]
fn parse_set(walker: &mut Walker<'_>) -> Result<Edn, Error> {
    let _ = walker.nibble_next(); // Consume the leading '{' char
    let mut set: BTreeSet<Edn> = BTreeSet::new();

    loop {
        match walker.peek_next() {
            Some('}') => {
                let _ = walker.nibble_next();
                return Ok(Edn::Set(Set::new(set)));
            }
            Some(_) => {
                let next = parse_internal(walker)?;
                if next != Edn::Empty {
                    set.insert(next);
                }
            }
            _ => {
                return Err(Error {
                    code: Code::UnexpectedEOF,
                    line: Some(walker.line),
                    column: Some(walker.column),
                    ptr: Some(walker.ptr),
                })
            }
        }
    }
}

#[inline]
#[cfg(not(feature = "sets"))]
const fn parse_set(walker: &Walker<'_>) -> Result<Edn, Error> {
    Err(Error {
        code: Code::NoFeatureSets,
        line: Some(walker.line),
        column: Some(walker.column),
        ptr: Some(walker.ptr),
    })
}

#[inline]
fn parse_tag(walker: &mut Walker<'_>) -> Result<Edn, Error> {
    let tag = walker.slurp_tag();
    Ok(Edn::Tagged(
        tag.to_string(),
        Box::new(parse_internal(walker)?),
    ))
}

#[inline]
fn parse_map(walker: &mut Walker<'_>) -> Result<Edn, Error> {
    let _ = walker.nibble_next(); // Consume the leading '{' char
    let mut map: BTreeMap<String, Edn> = BTreeMap::new();
    loop {
        match walker.peek_next() {
            Some('}') => {
                let _ = walker.nibble_next();
                return Ok(Edn::Map(Map::new(map)));
            }
            Some(n) => {
                if n == ']' || n == ')' {
                    return Err(Error {
                        code: Code::UnmatchedDelimiter(n),
                        line: Some(walker.line),
                        column: Some(walker.column),
                        ptr: Some(walker.ptr),
                    });
                }

                let key = parse_internal(walker)?;
                let val = parse_internal(walker)?;

                if key != Edn::Empty && val != Edn::Empty {
                    // Existing keys are considered an error
                    if map.insert(key.to_string(), val).is_some() {
                        return Err(Error {
                            code: Code::HashMapDuplicateKey,
                            line: Some(walker.line),
                            column: Some(walker.column),
                            ptr: Some(walker.ptr),
                        });
                    }
                }
            }
            _ => {
                return Err(Error {
                    code: Code::UnexpectedEOF,
                    line: Some(walker.line),
                    column: Some(walker.column),
                    ptr: Some(walker.ptr),
                })
            }
        }
    }
}

#[inline]
fn parse_vector(walker: &mut Walker<'_>) -> Result<Edn, Error> {
    let _ = walker.nibble_next(); // Consume the leading '[' char
    let mut vec = Vec::new();

    loop {
        match walker.peek_next() {
            Some(']') => {
                let _ = walker.nibble_next();
                return Ok(Edn::Vector(Vector::new(vec)));
            }
            Some(_) => {
                let next = parse_internal(walker)?;
                if next != Edn::Empty {
                    vec.push(next);
                }
            }
            _ => {
                return Err(Error {
                    code: Code::UnexpectedEOF,
                    line: Some(walker.line),
                    column: Some(walker.column),
                    ptr: Some(walker.ptr),
                })
            }
        }
    }
}

#[inline]
fn parse_list(walker: &mut Walker<'_>) -> Result<Edn, Error> {
    let _ = walker.nibble_next(); // Consume the leading '[' char
    let mut vec = Vec::new();

    loop {
        match walker.peek_next() {
            Some(')') => {
                let _ = walker.nibble_next();
                return Ok(Edn::List(List::new(vec)));
            }
            Some(_) => {
                let next = parse_internal(walker)?;
                if next != Edn::Empty {
                    vec.push(next);
                }
            }
            _ => {
                return Err(Error {
                    code: Code::UnexpectedEOF,
                    line: Some(walker.line),
                    column: Some(walker.column),
                    ptr: Some(walker.ptr),
                })
            }
        }
    }
}

#[inline]
fn edn_literal(literal: &str) -> Result<Edn, Code> {
    fn numeric(s: &str) -> bool {
        let (first, second) = {
            let mut s = s.chars();
            (s.next(), s.next())
        };

        if let Some(f) = first {
            if f.is_numeric() {
                return true;
            }

            if f == '-' || f == '+' {
                if let Some(s) = second {
                    if s.is_numeric() {
                        return true;
                    }
                }
            }
        }
        false
    }

    Ok(match literal {
        "nil" => Edn::Nil,
        "true" => Edn::Bool(true),
        "false" => Edn::Bool(false),
        "" => Edn::Empty,
        k if k.starts_with(':') => {
            if k.len() <= 1 {
                return Err(Code::InvalidKeyword);
            }
            Edn::Key(k.to_owned())
        }
        n if numeric(n) => return parse_number(n),
        _ => Edn::Symbol(literal.to_owned()),
    })
}

#[inline]
fn parse_char(lit: &str) -> Result<Edn, Code> {
    let lit = &lit[1..]; // ignore the leading '\\'
    match lit {
        "newline" => Ok(Edn::Char('\n')),
        "return" => Ok(Edn::Char('\r')),
        "tab" => Ok(Edn::Char('\t')),
        "space" => Ok(Edn::Char(' ')),
        c if c.len() == 1 => Ok(Edn::Char(c.chars().next().unwrap())),
        _ => Err(Code::InvalidChar),
    }
}

#[inline]
fn parse_number(lit: &str) -> Result<Edn, Code> {
    let mut chars = lit.chars();
    let (number, radix) = {
        let mut number = String::new();

        // The EDN spec allows for a redundant '+' symbol, we just ignore it.
        if let Some(n) = chars.next() {
            if n != '+' {
                number.push(n);
            }
        }

        for c in chars {
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
                    (number[1..index]).parse::<u8>()
                } else {
                    (number[0..index]).parse::<u8>()
                }
            };
            match radix {
                Ok(r) => {
                    // from_str_radix panics if radix is not in the range from 2 to 36
                    if !(2..=36).contains(&r) {
                        return Err(Code::InvalidRadix(Some(r)));
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
                Err(_) => {
                    return Err(Code::InvalidRadix(None));
                }
            }
        } else {
            (number, 10)
        }
    };

    if let Ok(n) = u64::from_str_radix(&number, radix.into()) {
        return Ok(Edn::UInt(n));
    }
    if let Ok(n) = i64::from_str_radix(&number, radix.into()) {
        return Ok(Edn::Int(n));
    }
    if let Ok(n) = number.parse::<f64>() {
        return Ok(Edn::Double(n.into()));
    }
    if let Some((n, d)) = num_den_from_slice(&number) {
        return Ok(Edn::Rational((n, d)));
    }

    Err(Code::InvalidNumber)
}

#[inline]
fn num_den_from_slice(slice: impl AsRef<str>) -> Option<(i64, u64)> {
    let slice = slice.as_ref();
    let index = slice.find('/');

    if let Some(i) = index {
        let (num, den) = slice.split_at(i); // This can't panic because the index is valid
        let num = num.parse::<i64>();
        let den = den[1..].parse::<u64>();

        if let (Ok(n), Ok(d)) = (num, den) {
            return Some((n, d));
        }
        return None;
    }
    None
}
