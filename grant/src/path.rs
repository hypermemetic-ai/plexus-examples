//! Paths are values, not strings. Text is how one is written and read at
//! the edge; it is parsed once, where it enters, and everything after works
//! on segments. A new form of segment (a keyed child, `shelf[3]`) is then a
//! change here, and nothing that compares, matches or orders paths changes.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathError {
    /// The byte the text went wrong at.
    pub at: usize,
    pub why: &'static str,
}

/// One segment of a method's path: an activation's, a child's or a method's
/// name, as the IR writes it — lower-case letters, digits, `-` and `_`.
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

/// Where a method is served: `studio.kiln.fire`. At least one segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(Vec<Name>);

impl Path {
    pub fn segments(&self) -> &[Name] {
        &self.0
    }
    /// The segments as text, for matching on: `["kiln", "fire"]`.
    pub fn names(&self) -> Vec<&str> {
        self.0.iter().map(Name::as_str).collect()
    }
    /// The method's own name: `fire`.
    pub fn method(&self) -> &Name {
        self.0.last().expect("a path has a segment")
    }
    /// Where the method's activation is served: `studio.kiln`.
    pub fn activation(&self) -> &[Name] {
        &self.0[..self.0.len() - 1]
    }
    /// The same method served under a parent: `kiln.fire` held at `studio`.
    pub fn under(&self, parent: &Name) -> Path {
        Path(std::iter::once(parent.clone()).chain(self.0.iter().cloned()).collect())
    }
}

/// Parse the segments of `text` split on `sep`, each by `seg`, naming the
/// byte where a segment went wrong.
fn segments<T>(text: &str, sep: char, seg: impl Fn(&str) -> Result<T, PathError>) -> Result<Vec<T>, PathError> {
    let mut at = 0;
    text.split(sep)
        .map(|s| {
            let start = at;
            at += s.len() + 1;
            seg(s).map_err(|e| PathError { at: start + e.at, why: e.why })
        })
        .collect()
}

impl FromStr for Path {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Path, PathError> {
        segments(s, '.', Name::new).map(Path)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.names().join("."))
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

/// A set of method paths, written as a path with `*` and `**`:
/// `studio.kiln.*`, `studio.**`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern(Vec<Step>);

impl Pattern {
    pub fn matches(&self, path: &Path) -> bool {
        let segs = path.segments();
        for (i, step) in self.0.iter().enumerate() {
            match step {
                Step::Rest => return segs.len() > i,
                Step::One if i < segs.len() => {}
                Step::Name(n) if segs.get(i) == Some(n) => {}
                _ => return false,
            }
        }
        segs.len() == self.0.len()
    }
}

impl FromStr for Pattern {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Pattern, PathError> {
        let steps = segments(s, '.', |seg| {
            Ok(match seg {
                "*" => Step::One,
                "**" => Step::Rest,
                n => Step::Name(Name::new(n)?),
            })
        })?;
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

    #[test]
    fn a_path_is_read_once_and_written_back_the_same() {
        let p: Path = "studio.kiln.fire".parse().unwrap();
        assert_eq!(p.names(), ["studio", "kiln", "fire"]);
        assert_eq!(p.method().as_str(), "fire");
        assert_eq!(p.to_string(), "studio.kiln.fire");
        assert_eq!(p.activation().len(), 2);
        let held: Path = "kiln.fire".parse::<Path>().unwrap().under(&Name::new("studio").unwrap());
        assert_eq!(held, p);
    }

    #[test]
    fn a_bad_path_names_where() {
        assert_eq!("studio..fire".parse::<Path>(), Err(PathError { at: 7, why: "an empty segment" }));
        assert_eq!("studio.Kiln".parse::<Path>().unwrap_err().at, 7);
        assert_eq!("studio.**.fire".parse::<Pattern>().unwrap_err(), PathError { at: 7, why: "`**` is the last segment" });
        assert!("studio.*.fire".parse::<Path>().is_err(), "a path is not a pattern");
    }

    #[test]
    fn patterns_match_segments() {
        let path = |s: &str| s.parse::<Path>().unwrap();
        let pat = |s: &str| s.parse::<Pattern>().unwrap();
        assert!(pat("studio.kiln.*").matches(&path("studio.kiln.fire")));
        assert!(!pat("studio.kiln.*").matches(&path("studio.kiln.log.list")));
        assert!(pat("studio.kiln.**").matches(&path("studio.kiln.log.list")));
        assert!(!pat("studio.kiln.**").matches(&path("studio.kiln")));
        assert!(pat("studio.*.fire").matches(&path("studio.kiln.fire")));
        assert!(!pat("studio.kiln").matches(&path("studio.kilns")), "a segment is compared whole");
        assert_eq!(pat("studio.*.**").to_string(), "studio.*.**");
    }
}
