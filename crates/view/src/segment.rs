use std::fmt;

use crate::{Field, FieldBuf};

/// Segment is a chunk of a `ViewPath`.
///
/// If you need an owned, allocated version, see `SegmentBuf`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Segment<'a> {
    Field(Field<'a>),
    Index(isize),
    Coalesce(Vec<Field<'a>>),
}

impl<'a> Segment<'a> {
    pub fn field(field: Field<'a>) -> Self {
        Self::Field(field)
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Segment::Field(_))
    }

    pub fn index(i: isize) -> Self {
        Self::Index(i)
    }

    pub fn is_index(&self) -> bool {
        matches!(self, Segment::Index(_))
    }

    pub fn coalesce(v: Vec<Field<'a>>) -> Self {
        Self::Coalesce(v)
    }

    pub fn is_coalesce(&self) -> bool {
        matches!(self, Segment::Coalesce(_))
    }

    pub fn to_segment_buf(&self) -> SegmentBuf {
        match self {
            Segment::Field(field) => SegmentBuf::field(field.to_field_buf()),
            Segment::Index(i) => SegmentBuf::index(*i),
            Segment::Coalesce(v) => {
                SegmentBuf::coalesce(v.iter().map(|field| field.to_field_buf()).collect())
            }
        }
    }
}

impl<'a> fmt::Display for Segment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Segment::Index(i) => write!(f, "{}", i),
            Segment::Field(field) => write!(f, "{}", field),
            Segment::Coalesce(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        }
    }
}

impl<'a> From<&'a str> for Segment<'a> {
    fn from(name: &'a str) -> Self {
        Self::Field(name.into())
    }
}

impl<'a> From<isize> for Segment<'a> {
    fn from(value: isize) -> Self {
        Self::index(value)
    }
}

impl<'a> From<Vec<Field<'a>>> for Segment<'a> {
    fn from(value: Vec<Field<'a>>) -> Self {
        Self::coalesce(value)
    }
}

impl<'a> From<&'a SegmentBuf> for Segment<'a> {
    fn from(v: &'a SegmentBuf) -> Self {
        v.as_segment()
    }
}

/// Segment is a chunk of a `ViewPath`.
///
/// This is the owned, allocated side of a `Segment`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SegmentBuf {
    Field(FieldBuf),
    Index(isize),
    Coalesce(Vec<FieldBuf>),
}

impl SegmentBuf {
    pub fn field(field: FieldBuf) -> Self {
        Self::Field(field)
    }

    pub fn is_field(&self) -> bool {
        matches!(self, SegmentBuf::Field(_))
    }

    pub fn index(i: isize) -> Self {
        Self::Index(i)
    }

    pub fn is_index(&self) -> bool {
        matches!(self, SegmentBuf::Index(_))
    }

    pub fn coalesce(v: Vec<FieldBuf>) -> Self {
        Self::Coalesce(v)
    }

    pub fn is_coalesce(&self) -> bool {
        matches!(self, SegmentBuf::Coalesce(_))
    }

    pub fn as_segment(&self) -> Segment<'_> {
        match self {
            SegmentBuf::Field(field) => Segment::field(field.as_field()),
            SegmentBuf::Index(i) => Segment::index(*i),
            SegmentBuf::Coalesce(v) => {
                Segment::coalesce(v.iter().map(|field| field.as_field()).collect())
            }
        }
    }
}

impl fmt::Display for SegmentBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            SegmentBuf::Index(i) => write!(f, "{}", i),
            SegmentBuf::Field(field) => write!(f, "{}", field),
            SegmentBuf::Coalesce(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        }
    }
}

impl From<String> for SegmentBuf {
    fn from(name: String) -> Self {
        Self::Field(name.into())
    }
}

impl From<&str> for SegmentBuf {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

impl From<isize> for SegmentBuf {
    fn from(value: isize) -> Self {
        Self::index(value)
    }
}

impl From<Vec<FieldBuf>> for SegmentBuf {
    fn from(value: Vec<FieldBuf>) -> Self {
        Self::coalesce(value)
    }
}

impl<'a> From<Segment<'a>> for SegmentBuf {
    fn from(value: Segment<'a>) -> Self {
        value.to_segment_buf()
    }
}
