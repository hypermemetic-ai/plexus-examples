//! Roles as expressions over a service's methods.
//!
//! The universe is the service's methods, each by its served path, each with
//! the groups its author put it in. An expression denotes a set of them, and
//! a role is a named expression. Every set is a bitset over the methods, so
//! a check is one bit, and two roles are compared exactly.
//!
//! ```text
//! expr   := term ("||" term)*
//! term   := factor ("&&" factor)*
//! factor := "!" factor | "(" expr ")" | atom
//! atom   := path | "group:" NAME | "role:" NAME
//! path   := a `Pattern`: SEG ("." SEG)*, SEG a name, "*" (one segment) or "**" (any depth, last only)
//! ```

use crate::path::{Path, Pattern};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// One method of the service, with its groups.
#[derive(Debug, Clone)]
pub struct MethodEntry {
    pub path: Path,
    pub groups: BTreeSet<String>,
}

/// Every method the service serves, in a fixed order: bit `i` is method `i`.
#[derive(Debug, Clone, Default)]
pub struct Methods(pub Vec<MethodEntry>);

impl Methods {
    pub fn add(&mut self, path: Path, groups: &[&str]) {
        self.0.push(MethodEntry { path, groups: groups.iter().map(|g| g.to_string()).collect() });
    }
    fn index(&self, path: &Path) -> Option<usize> {
        self.0.iter().position(|m| m.path == *path)
    }
}

/// A set of methods.
#[derive(Clone, PartialEq, Eq)]
pub struct Set(Vec<u64>);

impl Set {
    fn empty(n: usize) -> Set {
        Set(vec![0; n.div_ceil(64)])
    }
    fn all(n: usize) -> Set {
        let mut s = Set::empty(n);
        (0..n).for_each(|i| s.put(i));
        s
    }
    fn put(&mut self, i: usize) {
        self.0[i / 64] |= 1 << (i % 64);
    }
    pub fn has(&self, i: usize) -> bool {
        self.0[i / 64] >> (i % 64) & 1 == 1
    }
    pub fn and(&self, o: &Set) -> Set {
        Set(self.0.iter().zip(&o.0).map(|(a, b)| a & b).collect())
    }
    fn or(&self, o: &Set) -> Set {
        Set(self.0.iter().zip(&o.0).map(|(a, b)| a | b).collect())
    }
    fn not(&self, n: usize) -> Set {
        Set::all(n).and(&Set(self.0.iter().map(|a| !a).collect()))
    }
    /// Every method of `self` is in `o`.
    pub fn within(&self, o: &Set) -> bool {
        self.and(o) == *self
    }
    pub fn paths<'a>(&self, m: &'a Methods) -> Vec<&'a Path> {
        (0..m.0.len()).filter(|&i| self.has(i)).map(|i| &m.0[i].path).collect()
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Path(Pattern),
    Group(String),
    Role(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum RoleError {
    Parse { at: usize, expected: &'static str },
    UnknownRole(String),
    UnknownGroup(String),
    /// A role that names itself, through any chain.
    Cycle(Vec<String>),
    /// `!x` with nothing positive beside it: it would grant every method
    /// added to the service later. Write `a && !x`.
    Unbounded(String),
    /// A pattern that matches no method today: most likely a typo.
    Matches(Pattern),
}

// ---- parsing ----

pub fn parse(src: &str) -> Result<Expr, RoleError> {
    let mut p = Parser { s: src.as_bytes(), i: 0 };
    let e = p.expr()?;
    p.ws();
    if p.i != p.s.len() {
        return Err(RoleError::Parse { at: p.i, expected: "`&&`, `||` or the end" });
    }
    Ok(e)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, tok: &str) -> bool {
        self.ws();
        if self.s[self.i..].starts_with(tok.as_bytes()) {
            self.i += tok.len();
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Result<Expr, RoleError> {
        let mut e = self.term()?;
        while self.eat("||") {
            e = Expr::Or(Box::new(e), Box::new(self.term()?));
        }
        Ok(e)
    }
    fn term(&mut self) -> Result<Expr, RoleError> {
        let mut e = self.factor()?;
        while self.eat("&&") {
            e = Expr::And(Box::new(e), Box::new(self.factor()?));
        }
        Ok(e)
    }
    fn factor(&mut self) -> Result<Expr, RoleError> {
        if self.eat("!") {
            return Ok(Expr::Not(Box::new(self.factor()?)));
        }
        if self.eat("(") {
            let e = self.expr()?;
            return if self.eat(")") { Ok(e) } else { Err(RoleError::Parse { at: self.i, expected: "`)`" }) };
        }
        self.ws();
        let start = self.i;
        let word = self.word();
        if word.is_empty() {
            return Err(RoleError::Parse { at: self.i, expected: "a path, `group:` or `role:`" });
        }
        Ok(if let Some(g) = word.strip_prefix("group:") {
            Expr::Group(g.into())
        } else if let Some(r) = word.strip_prefix("role:") {
            Expr::Role(r.into())
        } else {
            Expr::Path(word.parse().map_err(|e: crate::path::PathError| RoleError::Parse { at: start + e.at, expected: e.why })?)
        })
    }
    fn word(&mut self) -> String {
        self.ws();
        let start = self.i;
        while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || b"-_.:*".contains(&self.s[self.i])) {
            self.i += 1;
        }
        String::from_utf8_lossy(&self.s[start..self.i]).into()
    }
}

// ---- meaning ----

/// The roles of one service: each a name and an expression, compiled once
/// against its methods. Defined in code by the macro's `roles(..)`, and at
/// run time by an admin; a stored definition wins over the code's.
pub struct Roles {
    methods: Methods,
    defs: BTreeMap<String, Expr>,
    sets: BTreeMap<String, Set>,
}

impl Roles {
    pub fn new(methods: Methods) -> Roles {
        Roles { methods, defs: BTreeMap::new(), sets: BTreeMap::new() }
    }

    pub fn methods(&self) -> &Methods {
        &self.methods
    }

    /// Define or redefine a role. Refused, with nothing changed, when it is
    /// malformed, names what is not there, is unbounded, or makes a cycle.
    pub fn define(&mut self, name: &str, src: &str) -> Result<(), RoleError> {
        let e = parse(src)?;
        if !bounded(&e) {
            return Err(RoleError::Unbounded(src.into()));
        }
        let mut defs = self.defs.clone();
        defs.insert(name.into(), e);
        let sets = compile_all(&self.methods, &defs)?;
        (self.defs, self.sets) = (defs, sets);
        Ok(())
    }

    /// The methods a role grants.
    pub fn set(&self, role: &str) -> Option<&Set> {
        self.sets.get(role)
    }

    /// Does `role` grant the method at `path`?
    pub fn grants(&self, role: &str, path: &Path) -> bool {
        match (self.sets.get(role), self.methods.index(path)) {
            (Some(s), Some(i)) => s.has(i),
            _ => false,
        }
    }

    /// The methods are recompiled when the service changes (a child
    /// mounted, an update applied): every role is meant again over the new
    /// set, and a definition that no longer holds is said, not dropped.
    pub fn remeasure(&mut self, methods: Methods) -> Result<(), RoleError> {
        let sets = compile_all(&methods, &self.defs)?;
        (self.methods, self.sets) = (methods, sets);
        Ok(())
    }
}

/// An expression is bounded when what it denotes lies within its positive
/// atoms: a method added later is granted only if one of them names it.
/// `!` may only take away from something beside it.
fn bounded(e: &Expr) -> bool {
    match e {
        Expr::Path(_) | Expr::Group(_) | Expr::Role(_) => true,
        Expr::Not(_) => false,
        Expr::Or(a, b) => bounded(a) && bounded(b),
        Expr::And(a, b) => bounded(a) || bounded(b),
    }
}

fn compile_all(methods: &Methods, defs: &BTreeMap<String, Expr>) -> Result<BTreeMap<String, Set>, RoleError> {
    let mut done = BTreeMap::new();
    for name in defs.keys() {
        compile_role(methods, defs, name, &mut Vec::new(), &mut done)?;
    }
    Ok(done)
}

fn compile_role(m: &Methods, defs: &BTreeMap<String, Expr>, name: &str, stack: &mut Vec<String>, done: &mut BTreeMap<String, Set>) -> Result<Set, RoleError> {
    if let Some(s) = done.get(name) {
        return Ok(s.clone());
    }
    if stack.iter().any(|s| s == name) {
        let mut c = stack.clone();
        c.push(name.into());
        return Err(RoleError::Cycle(c));
    }
    let e = defs.get(name).ok_or_else(|| RoleError::UnknownRole(name.into()))?;
    stack.push(name.into());
    let s = eval(m, defs, e, stack, done)?;
    stack.pop();
    done.insert(name.into(), s.clone());
    Ok(s)
}

fn eval(m: &Methods, defs: &BTreeMap<String, Expr>, e: &Expr, stack: &mut Vec<String>, done: &mut BTreeMap<String, Set>) -> Result<Set, RoleError> {
    let n = m.0.len();
    Ok(match e {
        Expr::Path(p) => {
            let mut s = Set::empty(n);
            m.0.iter().enumerate().filter(|(_, x)| p.matches(&x.path)).for_each(|(i, _)| s.put(i));
            if s == Set::empty(n) {
                return Err(RoleError::Matches(p.clone()));
            }
            s
        }
        Expr::Group(g) => {
            let mut s = Set::empty(n);
            m.0.iter().enumerate().filter(|(_, x)| x.groups.contains(g)).for_each(|(i, _)| s.put(i));
            if s == Set::empty(n) {
                return Err(RoleError::UnknownGroup(g.clone()));
            }
            s
        }
        Expr::Role(r) => compile_role(m, defs, r, stack, done)?,
        Expr::Not(a) => eval(m, defs, a, stack, done)?.not(n),
        Expr::And(a, b) => eval(m, defs, a, stack, done)?.and(&eval(m, defs, b, stack, done)?),
        Expr::Or(a, b) => eval(m, defs, a, stack, done)?.or(&eval(m, defs, b, stack, done)?),
    })
}
