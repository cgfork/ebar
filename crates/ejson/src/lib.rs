//! This module provides some useful help functions for JSON.
//!

use serde_json::Value;
use view::{Segment, SegmentBuf, ViewPath, ViewPathBuf};

pub fn nest_find_value<'a>(value: &'a Value, expect: &str) -> Option<Vec<ViewPathBuf>> {
    match nest_find_by(value, |v| match v {
        Value::Null => false,
        Value::Bool(b) => b.to_string().eq(expect),
        Value::Number(n) => n.to_string().eq(expect),
        Value::String(s) => s.eq(expect),
        Value::Array(_) => false,
        Value::Object(_) => false,
    }) {
        Some(values) => Some(values.into_iter().map(|s| s.0).collect()),
        None => None,
    }
}

pub fn nest_find_regex<'a>(
    value: &'a Value,
    expect: regex::Regex,
) -> Option<Vec<(ViewPathBuf, &'a Value)>> {
    nest_find_by(value, |v| match v {
        Value::Null => false,
        Value::Bool(b) => expect.is_match(&b.to_string()),
        Value::Number(n) => expect.is_match(&n.to_string()),
        Value::String(s) => expect.is_match(s),
        Value::Array(_) => false,
        Value::Object(_) => false,
    })
}

pub fn nest_find_by<'a>(
    value: &'a Value,
    predicate: impl Fn(&Value) -> bool + Clone,
) -> Option<Vec<(ViewPathBuf, &Value)>> {
    match value {
        Value::Null => None,
        Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            if predicate(value) {
                Some(vec![(ViewPathBuf::root(), value)])
            } else {
                None
            }
        }

        Value::Array(a) => {
            let mut ret = Vec::new();
            for (i, v) in a.iter().enumerate() {
                match nest_find_by(v, predicate.clone()) {
                    Some(paths) => {
                        for (mut path, vp) in paths {
                            path.push_front(SegmentBuf::Index(i as isize));
                            ret.push((path, vp));
                        }
                    }
                    None => {}
                }
            }
            if ret.len() > 0 {
                Some(ret)
            } else {
                None
            }
        }
        Value::Object(m) => {
            let mut ret = Vec::new();
            for (k, v) in m {
                match nest_find_by(v, predicate.clone()) {
                    Some(paths) => {
                        for (mut path, vp) in paths {
                            path.push_front(SegmentBuf::Field(k.as_str().into()));
                            ret.push((path, vp));
                        }
                    }
                    None => {}
                }
            }
            if ret.len() > 0 {
                Some(ret)
            } else {
                None
            }
        }
    }
}

pub fn search_path<'a>(value: &'a Value, path: ViewPath<'a>) -> Option<&'a Value> {
    let mut value = value;
    for seg in path.into_iter() {
        match search_segment(value, seg) {
            Some(v) => value = v,
            None => return None,
        }
    }
    Some(value)
}

pub fn search_segment<'a>(value: &'a Value, seg: Segment<'a>) -> Option<&'a Value> {
    match seg {
        Segment::Field(f) => match value {
            Value::Object(m) => m.get(f.as_str()),
            _ => None,
        },
        Segment::Index(i) => match value {
            Value::Array(a) => {
                let i = i as usize;
                if i < a.len() {
                    Some(&a[i])
                } else {
                    None
                }
            }
            _ => None,
        },
        Segment::Coalesce(c) => match value {
            Value::Object(m) => {
                for f in c {
                    let v = m.get(f.as_str());
                    if v.is_some() {
                        return v;
                    }
                }
                None
            }
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
