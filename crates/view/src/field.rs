use std::{
    fmt::{self, Display},
    cell::RefCell,
};

use regex::Regex;

thread_local! {
    static VALID_FIELD: RefCell<Regex> =
       RefCell::new( Regex::new("^[0-9]*[a-zA-Z_][0-9a-zA-Z_]*$").unwrap());
}

/// A valid fieldname can contain alphanumeric characters and an underscore.
/// It may start with a number, but has to consist of more than just a number.
/// Fields that have other characters can be used, but need to be quoted.
pub(crate) fn is_valid_field_name(name: &str) -> bool {
    VALID_FIELD.with(|r| r.borrow().is_match(name))
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field<'a> {
    pub name: &'a str,
    // This is a optimization to avoid having to scan for escapes.
    pub requires_quoting: bool,
}

impl<'a> Field<'a> {
    pub fn to_field_buf(&self) -> FieldBuf {
        FieldBuf {
            name: self.name.to_string(),
            requires_quoting: self.requires_quoting,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl<'a> Display for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.requires_quoting {
            write!(f, r#""{}""#, self.name)
        } else {
            write!(f, r#"{}"#, self.name)
        }
    }
}

impl<'a> From<&'a str> for Field<'a> {
    fn from(mut name: &'a str) -> Self {
        let mut requires_quoting = false;

        if name.starts_with('\"') && name.ends_with('\"') {
            let len = name.len();
            name = &name[1..len - 1];
            requires_quoting = true;
        } else if !is_valid_field_name(name) {
            requires_quoting = true;
        }

        Self {
            name,
            requires_quoting,
        }
    }
}

impl<'a> From<&'a FieldBuf> for Field<'a> {
    fn from(fb: &'a FieldBuf) -> Self {
        Self {
            name: &fb.name,
            requires_quoting: fb.requires_quoting,
        }
    }
}

/// This is the owned, allocated side of a `Field.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldBuf {
    pub name: String,
    // This is a optimization to avoid having to scan for escapes.
    pub requires_quoting: bool,
}

impl FieldBuf {
    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn as_field(&self) -> Field<'_> {
        Field {
            name: &self.name,
            requires_quoting: self.requires_quoting,
        }
    }
}

impl Display for FieldBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.requires_quoting {
            write!(f, r#""{}""#, self.name)
        } else {
            write!(f, r#"{}"#, self.name)
        }
    }
}

impl From<String> for FieldBuf {
    fn from(mut name: String) -> Self {
        let mut requires_quoting = false;

        if name.starts_with('\"') && name.ends_with('\"') {
            // There is unfortunately no way to make an owned substring of a string.
            // So we have to take a slice and clone it.
            let len = name.len();
            name = name[1..len - 1].to_string();
            requires_quoting = true;
        } else if !is_valid_field_name(&name) {
            requires_quoting = true
        }

        Self {
            name,
            requires_quoting,
        }
    }
}

impl From<&str> for FieldBuf {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl quickcheck::Arbitrary for FieldBuf {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let chars = (32u8..90).map(|c| c as char).collect::<Vec<_>>();
        let len = u32::arbitrary(g) % 100 + 1;
        let name = (0..len)
            .map(|_| chars[usize::arbitrary(g) % chars.len()])
            .collect::<String>()
            .replace(r#"""#, r#"\""#);
        FieldBuf::from(name)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.name
                .shrink()
                .filter(|name| !name.is_empty())
                .map(|name| {
                    let name = name.replace(r#"""#, r#"/""#);
                    FieldBuf::from(name)
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::FieldBuf;

    #[test]
    fn test_field_buf_quoted() {
        let field: FieldBuf = "foo".into();
        assert_eq!(
            FieldBuf {
                name: "foo".into(),
                requires_quoting: false,
            },
            field
        );
    }

    #[test]
    fn test_field_buf_quickcheck() {
        fn test_field_buf(field_buf: FieldBuf) -> quickcheck::TestResult {
            let field_buf2: FieldBuf = field_buf.name.as_str().into();
            assert_eq!(field_buf, field_buf2);
            quickcheck::TestResult::passed()
        }
        quickcheck::QuickCheck::new()
            .tests(1_000)
            .max_tests(2_000)
            .quickcheck(test_field_buf as fn(FieldBuf) -> quickcheck::TestResult)
    }
}
