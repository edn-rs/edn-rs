use core::fmt::{self, Debug};
use core::{convert, num, str};

pub struct Error {
    pub code: Code,
    /// Counting from 1.
    pub line: Option<usize>,
    /// This is a utf-8 char count. Counting from 1.
    pub column: Option<usize>,
    /// This is a pointer into the str trying to be parsed, not a utf-8 char offset
    pub ptr: Option<usize>,
}

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Code {
    /// Parse errors
    HashMapDuplicateKey,
    InvalidChar,
    InvalidEscape,
    InvalidKeyword,
    InvalidNumber,
    InvalidRadix(Option<u8>),
    UnexpectedEOF,
    UnmatchedDelimiter(char),

    /// Feature errors
    NoFeatureSets,

    /// Deserialize errors
    Convert(&'static str),

    /// Type conversion errors
    TryFromInt(num::TryFromIntError),
    #[doc(hidden)]
    Infallable(), // Makes the compiler happy for converting u64 to u64 and i64 to i64
}

impl Error {
    pub(crate) const fn deserialize(conv_type: &'static str) -> Self {
        Self {
            code: Code::Convert(conv_type),
            line: None,
            column: None,
            ptr: None,
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EdnError {{ code: {:?}, line: {:?}, column: {:?}, ptr: {:?} }}",
            self.code, self.line, self.column, self.ptr
        )
    }
}

impl From<convert::Infallible> for Error {
    fn from(_: convert::Infallible) -> Self {
        Self {
            code: Code::Infallable(),
            line: None,
            column: None,
            ptr: None,
        }
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(e: num::TryFromIntError) -> Self {
        Self {
            code: Code::TryFromInt(e),
            line: None,
            column: None,
            ptr: None,
        }
    }
}
