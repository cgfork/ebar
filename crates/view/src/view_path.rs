use core::fmt;
use std::{
    borrow::Cow,
    collections::VecDeque,
    ops::{Index, IndexMut},
};

use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};

use crate::{Field, FieldBuf, Segment, SegmentBuf};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewPath<'a> {
    segments: VecDeque<Segment<'a>>,
}

impl<'a> fmt::Display for ViewPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut peeker = self.segments.iter().peekable();
        while let Some(segment) = peeker.next() {
            let maybe_next = peeker
                .peek()
                .map(|next| next.is_field() || next.is_coalesce())
                .unwrap_or(false);
            match (segment, maybe_next) {
                (Segment::Field(_), true) => write!(f, r#"{}."#, segment)?,
                (Segment::Field(_), false) => write!(f, "{}", segment)?,
                (Segment::Index(_), true) => write!(f, r#"[{}]."#, segment)?,
                (Segment::Index(_), false) => write!(f, "[{}]", segment)?,
                (Segment::Coalesce(_), true) => write!(f, r#"{}."#, segment)?,
                (Segment::Coalesce(_), false) => write!(f, "{}", segment)?,
            }
        }
        Ok(())
    }
}

impl<'a> ViewPath<'a> {
    pub fn root() -> Self {
        Self {
            segments: VecDeque::new(),
        }
    }

    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Segment<'a>> {
        self.segments.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Segment<'a>> {
        self.segments.get_mut(index)
    }

    pub fn push_back(&mut self, segment: impl Into<Segment<'a>>) {
        self.segments.push_back(segment.into())
    }

    pub fn pop_back(&mut self) -> Option<Segment<'a>> {
        self.segments.pop_back()
    }

    pub fn push_front(&mut self, segment: impl Into<Segment<'a>>) {
        self.segments.push_front(segment.into())
    }

    pub fn pop_front(&mut self) -> Option<Segment<'a>> {
        self.segments.pop_front()
    }

    pub fn extend(&mut self, other: Self) {
        self.segments.extend(other.segments)
    }

    pub fn parse_str(input: &'a str) -> Result<Self, crate::Error> {
        crate::parser::parse_view_path(input).map_err(crate::Error::InvalidPath)
    }

    pub fn starts_with(&self, needle: &ViewPath<'a>) -> bool {
        needle.iter().zip(&self.segments).all(|(n, s)| n == s)
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, Segment<'a>> {
        self.segments.iter()
    }

    pub fn into_buf(self) -> ViewPathBuf {
        ViewPathBuf::from(self)
    }
}

impl<'a> IntoIterator for ViewPath<'a> {
    type Item = Segment<'a>;
    type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}

impl<'a> AsRef<ViewPath<'a>> for ViewPath<'a> {
    fn as_ref(&self) -> &ViewPath<'a> {
        self
    }
}

impl<'a> From<&'a str> for ViewPath<'a> {
    fn from(input: &'a str) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(Segment::from(input));
        Self { segments }
    }
}

impl<'a> From<isize> for ViewPath<'a> {
    fn from(input: isize) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(Segment::from(input));
        Self { segments }
    }
}

impl<'a> From<&'a String> for ViewPath<'a> {
    fn from(input: &'a String) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(Segment::from(input.as_str()));
        Self { segments }
    }
}

impl<'a> From<Segment<'a>> for ViewPath<'a> {
    fn from(input: Segment<'a>) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(input);
        Self { segments }
    }
}

impl<'a> From<VecDeque<Segment<'a>>> for ViewPath<'a> {
    fn from(segments: VecDeque<Segment<'a>>) -> Self {
        Self { segments }
    }
}

impl<'collection: 'item, 'item> From<&'collection [SegmentBuf]> for ViewPath<'item> {
    fn from(segments: &'collection [SegmentBuf]) -> Self {
        Self {
            segments: segments.iter().map(Segment::from).collect(),
        }
    }
}

impl<'collection: 'item, 'item> From<&'collection VecDeque<SegmentBuf>> for ViewPath<'item> {
    fn from(segments: &'collection VecDeque<SegmentBuf>) -> Self {
        Self {
            segments: segments.iter().map(Segment::from).collect(),
        }
    }
}

impl<'a> From<Field<'a>> for ViewPath<'a> {
    fn from(field: Field<'a>) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(Segment::Field(field));
        Self { segments }
    }
}

impl<'a> From<&'a ViewPathBuf> for ViewPath<'a> {
    fn from(lookup_buf: &'a ViewPathBuf) -> Self {
        Self::from(&lookup_buf.segments)
    }
}

impl<'a> Serialize for ViewPath<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&*self.to_string())
    }
}

impl<'de> Deserialize<'de> for ViewPath<'de> {
    fn deserialize<D>(deserializer: D) -> Result<ViewPath<'de>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ViewPathVisitor {
            _marker: Default::default(),
        })
    }
}

impl<'a> Index<usize> for ViewPath<'a> {
    type Output = Segment<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        self.segments.index(index)
    }
}

impl<'a> IndexMut<usize> for ViewPath<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.segments.index_mut(index)
    }
}

struct ViewPathVisitor<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'de> Visitor<'de> for ViewPathVisitor<'de> {
    type Value = ViewPath<'de>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("not a view path str")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ViewPath::parse_str(value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewPathBuf {
    segments: VecDeque<SegmentBuf>,
}

impl fmt::Display for ViewPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut peeker = self.segments.iter().peekable();
        while let Some(segment) = peeker.next() {
            let maybe_next = peeker
                .peek()
                .map(|next| next.is_field() || next.is_coalesce())
                .unwrap_or(false);
            match (segment, maybe_next) {
                (SegmentBuf::Field(_), true) => write!(f, r#"{}."#, segment)?,
                (SegmentBuf::Field(_), false) => write!(f, "{}", segment)?,
                (SegmentBuf::Index(_), true) => write!(f, r#"[{}]."#, segment)?,
                (SegmentBuf::Index(_), false) => write!(f, "[{}]", segment)?,
                (SegmentBuf::Coalesce(_), true) => write!(f, r#"{}."#, segment)?,
                (SegmentBuf::Coalesce(_), false) => write!(f, "{}", segment)?,
            }
        }
        Ok(())
    }
}

impl ViewPathBuf {
    pub fn root() -> Self {
        Self {
            segments: VecDeque::new(),
        }
    }

    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&SegmentBuf> {
        self.segments.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut SegmentBuf> {
        self.segments.get_mut(index)
    }

    pub fn push_back(&mut self, segment: impl Into<SegmentBuf>) {
        self.segments.push_back(segment.into())
    }

    pub fn pop_back(&mut self) -> Option<SegmentBuf> {
        self.segments.pop_back()
    }

    pub fn push_front(&mut self, segment: impl Into<SegmentBuf>) {
        self.segments.push_front(segment.into())
    }

    pub fn pop_front(&mut self) -> Option<SegmentBuf> {
        self.segments.pop_front()
    }

    pub fn extend(&mut self, other: Self) {
        self.segments.extend(other.segments)
    }

    pub fn parse_str(input: &str) -> Result<Self, crate::Error> {
        ViewPath::parse_str(input).map(|vp| vp.into_buf())
    }

    pub fn starts_with(&self, needle: &ViewPathBuf) -> bool {
        needle.iter().zip(&self.segments).all(|(n, s)| n == s)
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, SegmentBuf> {
        self.segments.iter()
    }

    pub fn as_view_path(&self) -> ViewPath<'_> {
        ViewPath::from(self)
    }
}

impl std::str::FromStr for ViewPathBuf {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, crate::Error> {
        Self::parse_str(input)
    }
}

impl IntoIterator for ViewPathBuf {
    type Item = SegmentBuf;
    type IntoIter = std::collections::vec_deque::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}

impl From<VecDeque<SegmentBuf>> for ViewPathBuf {
    fn from(segments: VecDeque<SegmentBuf>) -> Self {
        ViewPathBuf { segments }
    }
}

impl From<String> for ViewPathBuf {
    fn from(input: String) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(SegmentBuf::from(input));
        ViewPathBuf { segments }
    }
}

impl From<Cow<'_, str>> for ViewPathBuf {
    fn from(input: Cow<'_, str>) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(SegmentBuf::from(input.as_ref()));
        ViewPathBuf { segments }
    }
}

impl From<SegmentBuf> for ViewPathBuf {
    fn from(input: SegmentBuf) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(input);
        ViewPathBuf { segments }
    }
}

impl From<isize> for ViewPathBuf {
    fn from(input: isize) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(SegmentBuf::index(input));
        ViewPathBuf { segments }
    }
}

impl From<&str> for ViewPathBuf {
    fn from(input: &str) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(SegmentBuf::from(input.to_owned()));
        ViewPathBuf { segments }
    }
}

impl From<FieldBuf> for ViewPathBuf {
    fn from(field: FieldBuf) -> Self {
        let mut segments = VecDeque::with_capacity(1);
        segments.push_back(SegmentBuf::Field(field));
        Self { segments }
    }
}

impl<'a> From<ViewPath<'a>> for ViewPathBuf {
    fn from(v: ViewPath<'a>) -> Self {
        let segments = v
            .segments
            .into_iter()
            .map(|f| f.to_segment_buf())
            .collect::<VecDeque<_>>();
        ViewPathBuf::from(segments)
    }
}

impl Serialize for ViewPathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&*self.to_string())
    }
}

impl<'de> Deserialize<'de> for ViewPathBuf {
    fn deserialize<D>(deserializer: D) -> Result<ViewPathBuf, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ViewPathBufVisitor)
    }
}

impl Index<usize> for ViewPathBuf {
    type Output = SegmentBuf;

    fn index(&self, index: usize) -> &Self::Output {
        self.segments.index(index)
    }
}

impl IndexMut<usize> for ViewPathBuf {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.segments.index_mut(index)
    }
}

struct ViewPathBufVisitor;

impl<'de> Visitor<'de> for ViewPathBufVisitor {
    type Value = ViewPathBuf;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("not a view path str")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ViewPathBuf::parse_str(value).map_err(de::Error::custom)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ViewPathBuf::parse_str(value).map_err(de::Error::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ViewPathBuf::parse_str(&value).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::ViewPathBuf;

    #[test]
    fn test_parse_str() {
        let mut view_path = ViewPathBuf::parse_str("a.b.c.d[0]").unwrap();
        let a = view_path.pop_front().unwrap();
        assert_eq!(a.to_string(), "a".to_string());
        let b = view_path.pop_front().unwrap();
        assert_eq!(b.to_string(), "b".to_string());
        let c = view_path.pop_front().unwrap();
        assert_eq!(c.to_string(), "c".to_string());
        let d = view_path.pop_front().unwrap();
        assert_eq!(d.to_string(), "d".to_string());
        let e = view_path.pop_front().unwrap();
        assert_eq!(true, e.is_index());
    }
}
