//! Paths are values, not strings. Text is how one is written and read at
//! the edge; it is parsed once, where it enters, and everything after works
//! on segments.
//!
//! A segment is a name, and may carry keys: what the caller gives to open
//! that level. `studio[tenant=acme].kiln[3].fire` opens the studio for a
//! tenant, opens kiln 3 in it, and fires. The path is the whole address of
//! the call: nothing about it is kept on the server between calls.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathError {
    /// The byte the text went wrong at.
    pub at: usize,
    pub why: &'static str,
}

/// A name, as the IR writes one: lower-case letters, digits, `-` and `_`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(Box<str>);

impl Name {
    pub fn new(s: &str) -> Result<Name, PathError> {
        if s.is_empty() {
            return Err(PathError { at: 0, why: "an empty segment" });
        }
        match s.bytes().position(|b| !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')) {
            Some(at) => Err(PathError { at, why: "a name is lower-case letters, digits, `-` and `_`" }),
            None => Ok(Name(s.into())),
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A key's value: an id, as text. Anything but `[`, `]`, `,`, `=` and
/// control characters; the level it opens reads it as its own type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Value(Box<str>);

impl Value {
    pub fn new(s: &str) -> Result<Value, PathError> {
        if s.is_empty() {
            return Err(PathError { at: 0, why: "an empty key" });
        }
        match s.bytes().position(|b| b.is_ascii_control() || b"[],=".contains(&b)) {
            Some(at) => Err(PathError { at, why: "a key holds no `[`, `]`, `,`, `=` or control character" }),
            None => Ok(Value(s.into())),
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a segment was opened with.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Keys {
    /// `kiln`
    #[default]
    None,
    /// `kiln[3]`: the one key a child per key takes.
    One(Value),
    /// `studio[tenant=acme,season=2]`: a context's fields, by name.
    Named(Vec<(Name, Value)>),
}

impl Keys {
    pub fn one(&self) -> Option<&Value> {
        match self {
            Keys::One(v) => Some(v),
            _ => None,
        }
    }
    pub fn named(&self, name: &str) -> Option<&Value> {
        match self {
            Keys::Named(ks) => ks.iter().find(|(n, _)| n.as_str() == name).map(|(_, v)| v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Segment {
    pub name: Name,
    pub keys: Keys,
}

/// The address of a call: `studio[tenant=acme].kiln[3].fire`. At least one
/// segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(Vec<Segment>);

impl Path {
    /// A path of one segment, to build on: what a client's handle starts as.
    pub fn root(name: Name) -> Path {
        Path(vec![Segment { name, keys: Keys::None }])
    }
    /// One level down.
    pub fn then(mut self, name: Name) -> Path {
        self.0.push(Segment { name, keys: Keys::None });
        self
    }
    /// The last segment opened with one key: `kiln` to `kiln[3]`.
    pub fn at(mut self, key: Value) -> Path {
        self.0.last_mut().expect("a path has a segment").keys = Keys::One(key);
        self
    }
    /// The last segment opened with a named key, added to any it has.
    pub fn with(mut self, name: Name, value: Value) -> Path {
        let last = self.0.last_mut().expect("a path has a segment");
        match &mut last.keys {
            Keys::Named(ks) => ks.push((name, value)),
            keys => *keys = Keys::Named(vec![(name, value)]),
        }
        self
    }

    pub fn segments(&self) -> &[Segment] {
        &self.0
    }
    /// The segments' names, for matching on: `["studio", "kiln", "fire"]`.
    pub fn names(&self) -> Vec<&str> {
        self.0.iter().map(|s| s.name.as_str()).collect()
    }
    /// The method's own name: `fire`.
    pub fn method(&self) -> &Name {
        &self.0.last().expect("a path has a segment").name
    }
    /// The path with no keys: `studio.kiln.fire`. Which method this is,
    /// whatever data it is called on. Roles and grants see only this.
    pub fn shape(&self) -> Path {
        Path(self.0.iter().map(|s| Segment { name: s.name.clone(), keys: Keys::None }).collect())
    }
    /// The first `n` segments, keys kept: the address of one opened level.
    pub fn prefix(&self, n: usize) -> Path {
        Path(self.0[..n].to_vec())
    }
}

fn shifted<T>(r: Result<T, PathError>, by: usize) -> Result<T, PathError> {
    r.map_err(|e| PathError { at: by + e.at, why: e.why })
}

fn keys(text: &str, at: usize) -> Result<Keys, PathError> {
    let mut named = Vec::new();
    let mut offset = at;
    for part in text.split(',') {
        match part.split_once('=') {
            None if named.is_empty() && !text.contains(',') => return Ok(Keys::One(shifted(Value::new(part), offset)?)),
            None => return Err(PathError { at: offset, why: "several keys are each `name=value`" }),
            Some((n, v)) => named.push((shifted(Name::new(n), offset)?, shifted(Value::new(v), offset + n.len() + 1)?)),
        }
        offset += part.len() + 1;
    }
    Ok(Keys::Named(named))
}

impl FromStr for Path {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Path, PathError> {
        let (mut segs, mut i) = (Vec::new(), 0);
        loop {
            let end = s[i..].find(['.', '[']).map_or(s.len(), |n| i + n);
            let name = shifted(Name::new(&s[i..end]), i)?;
            i = end;
            let mut k = Keys::None;
            if s[i..].starts_with('[') {
                let close = s[i..].find(']').map(|n| i + n).ok_or(PathError { at: i, why: "a `[` with no `]`" })?;
                k = keys(&s[i + 1..close], i + 1)?;
                i = close + 1;
            }
            segs.push(Segment { name, keys: k });
            match s[i..].chars().next() {
                None => return Ok(Path(segs)),
                Some('.') => i += 1,
                Some(_) => return Err(PathError { at: i, why: "a `.` or the end after `]`" }),
            }
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, s) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{}", s.name.as_str())?;
            match &s.keys {
                Keys::None => {}
                Keys::One(v) => write!(f, "[{}]", v.as_str())?,
                Keys::Named(ks) => {
                    let parts: Vec<String> = ks.iter().map(|(n, v)| format!("{}={}", n.as_str(), v.as_str())).collect();
                    write!(f, "[{}]", parts.join(","))?;
                }
            }
        }
        Ok(())
    }
}

/// One step of a pattern over method paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Name(Name),
    /// `*`: any one segment.
    One,
    /// `**`: one segment or more; only last.
    Rest,
}

/// A set of methods, written as a path with `*` and `**`: `studio.kiln.*`,
/// `studio.**`. A pattern has no keys and reads none: it says which
/// methods, never which data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern(Vec<Step>);

impl Pattern {
    pub fn matches(&self, path: &Path) -> bool {
        let segs = path.segments();
        for (i, step) in self.0.iter().enumerate() {
            match step {
                Step::Rest => return segs.len() > i,
                Step::One if i < segs.len() => {}
                Step::Name(n) if segs.get(i).is_some_and(|s| s.name == *n) => {}
                _ => return false,
            }
        }
        segs.len() == self.0.len()
    }
}

impl FromStr for Pattern {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Pattern, PathError> {
        let mut at = 0;
        let steps = s
            .split('.')
            .map(|seg| {
                let start = at;
                at += seg.len() + 1;
                Ok(match seg {
                    "*" => Step::One,
                    "**" => Step::Rest,
                    n => Step::Name(shifted(Name::new(n), start)?),
                })
            })
            .collect::<Result<Vec<_>, PathError>>()?;
        if let Some(i) = steps.iter().position(|s| *s == Step::Rest).filter(|&i| i + 1 != steps.len()) {
            let at = s.split('.').take(i).map(|x| x.len() + 1).sum();
            return Err(PathError { at, why: "`**` is the last segment" });
        }
        Ok(Pattern(steps))
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let words: Vec<&str> = self.0.iter().map(|s| match s {
            Step::Name(n) => n.as_str(),
            Step::One => "*",
            Step::Rest => "**",
        }).collect();
        write!(f, "{}", words.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(s: &str) -> Path {
        s.parse().unwrap()
    }

    #[test]
    fn a_path_is_read_once_and_written_back_the_same() {
        for text in ["studio.kiln.fire", "studio[tenant=acme].kiln[3].fire", "studio[tenant=acme,season=2].status", "notes[ada@example.com].list"] {
            assert_eq!(path(text).to_string(), text);
        }
        let p = path("studio[tenant=acme].kiln[3].fire");
        assert_eq!(p.names(), ["studio", "kiln", "fire"]);
        assert_eq!(p.method().as_str(), "fire");
        assert_eq!(p.segments()[0].keys.named("tenant").unwrap().as_str(), "acme");
        assert_eq!(p.segments()[1].keys.one().unwrap().as_str(), "3");
        assert_eq!(p.segments()[2].keys, Keys::None);
    }

    #[test]
    fn the_shape_is_the_method_and_the_prefix_is_a_level() {
        let p = path("studio[tenant=acme].kiln[3].fire");
        assert_eq!(p.shape(), path("studio.kiln.fire"));
        assert_eq!(p.prefix(2), path("studio[tenant=acme].kiln[3]"));
        assert_ne!(p.prefix(2), path("studio[tenant=acme].kiln[4]"), "another kiln is another address");
    }

    #[test]
    fn a_handle_is_built_from_segments_never_from_text() {
        let n = |s: &str| Name::new(s).unwrap();
        let v = |s: &str| Value::new(s).unwrap();
        let built = Path::root(n("studio")).with(n("tenant"), v("acme")).then(n("kiln")).at(v("3")).then(n("fire"));
        assert_eq!(built, path("studio[tenant=acme].kiln[3].fire"));
        assert!(Value::new("3].drain").is_err(), "a key cannot close its own bracket");
    }

    #[test]
    fn a_bad_path_names_where() {
        let bad = |s: &str| s.parse::<Path>().unwrap_err();
        assert_eq!(bad("studio..fire"), PathError { at: 7, why: "an empty segment" });
        assert_eq!(bad("studio.Kiln").at, 7);
        assert_eq!(bad("studio.kiln[3.fire"), PathError { at: 11, why: "a `[` with no `]`" });
        assert_eq!(bad("studio.kiln[3]fire"), PathError { at: 14, why: "a `.` or the end after `]`" });
        assert_eq!(bad("studio.kiln[]").why, "an empty key");
        assert_eq!(bad("studio[tenant=acme,2].status"), PathError { at: 19, why: "several keys are each `name=value`" });
        assert!("studio.*.fire".parse::<Path>().is_err(), "a path is not a pattern");
        assert_eq!("studio.**.fire".parse::<Pattern>().unwrap_err(), PathError { at: 7, why: "`**` is the last segment" });
        assert!("studio.kiln[3].*".parse::<Pattern>().is_err(), "a pattern names methods, never data");
    }

    #[test]
    fn patterns_match_names_and_read_no_keys() {
        let pat = |s: &str| s.parse::<Pattern>().unwrap();
        assert!(pat("studio.kiln.*").matches(&path("studio.kiln.fire")));
        assert!(pat("studio.kiln.*").matches(&path("studio[tenant=acme].kiln[3].fire")));
        assert!(!pat("studio.kiln.*").matches(&path("studio.kiln.log.list")));
        assert!(pat("studio.kiln.**").matches(&path("studio.kiln.log.list")));
        assert!(!pat("studio.kiln.**").matches(&path("studio.kiln")));
        assert!(pat("studio.*.fire").matches(&path("studio.kiln.fire")));
        assert!(!pat("studio.kiln").matches(&path("studio.kilns")), "a segment is compared whole");
        assert_eq!(pat("studio.*.**").to_string(), "studio.*.**");
    }
}
