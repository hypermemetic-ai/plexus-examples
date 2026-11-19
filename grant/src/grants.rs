//! Who may call which method. Plexus decides up to the method's invocation;
//! what a caller may do to the data inside it (this kiln, that firing) is
//! the application's policy, decided in the method with the user in hand.
//!
//! A grant is `(who, role)`. A call is let in when some grant the caller
//! holds, directly, through a team or as everyone, has a role whose set
//! holds the method. That is the whole rule:
//!
//! ```text
//! allow(P, M) :- holds(P, W), grant(W, Role), in(M, Role).
//! holds(P, P).   holds(P, T) :- member(P, T).   holds(P, everyone) :- signed_in(P).
//! ```

use crate::call::{Principal, Refusal};
use crate::path::Path;
use crate::roles::{Roles, Set};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Who {
    One(Principal),
    Team(String),
    Everyone,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Grant {
    pub who: Who,
    pub role: String,
}

/// One service's grants and teams. A service keeps this in its own store;
/// a paired service is a principal in it like anyone.
#[derive(Default)]
pub struct Grants {
    pub grants: BTreeSet<Grant>,
    pub teams: BTreeMap<String, BTreeSet<Principal>>,
}

impl Grants {
    pub fn grant(&mut self, who: Who, role: &str) {
        self.grants.insert(Grant { who, role: role.into() });
    }

    fn holds(&self, p: &Principal, who: &Who) -> bool {
        match who {
            Who::One(q) => p == q,
            Who::Team(t) => self.teams.get(t).is_some_and(|m| m.contains(p)),
            Who::Everyone => true,
        }
    }

    /// Decide one call, before the method runs. The refusal names the
    /// method that was needed.
    pub fn decide(&self, roles: &Roles, p: Option<&Principal>, method: &Path) -> Result<(), Refusal> {
        let p = p.ok_or(Refusal::SignIn)?;
        let ok = self.grants.iter().any(|g| self.holds(p, &g.who) && roles.grants(&g.role, method));
        if ok { Ok(()) } else { Err(Refusal::Forbidden { grant: method.clone() }) }
    }

    /// Every method `p` may call: the union of its roles' sets. What a
    /// client asks to draw only the buttons a person may press; whether a
    /// button does anything to this kiln is the application's to say.
    pub fn reach(&self, roles: &Roles, p: &Principal) -> Vec<Path> {
        let mut out = BTreeSet::new();
        for g in self.grants.iter().filter(|g| self.holds(p, &g.who)) {
            if let Some(s) = roles.set(&g.role) {
                out.extend(s.paths(roles.methods()).into_iter().cloned());
            }
        }
        out.into_iter().collect()
    }
}

/// One link of a delegation: `from` lets `to` call for it, within a set of
/// methods, until a time. Signed by `from` (not drawn here). What the far
/// application then lets the call do to its data is its own policy, asked
/// about the first issuer.
#[derive(Debug, Clone)]
pub struct Link {
    pub from: Principal,
    pub to: Principal,
    pub methods: Set,
    pub until: u64,
}

/// What a chain of links lets its last holder call for its first issuer:
/// the meet of every link. Each link can only take away. Refused when the
/// chain is broken (a link not from the one before's `to`) or expired.
pub fn authority(chain: &[Link], now: u64) -> Option<(Principal, Set, Principal)> {
    let first = chain.first()?;
    if first.until < now {
        return None;
    }
    let (mut methods, mut holder) = (first.methods.clone(), first.to.clone());
    for l in &chain[1..] {
        if l.from != holder || l.until < now {
            return None;
        }
        methods = methods.and(&l.methods);
        holder = l.to.clone();
    }
    Some((first.from.clone(), methods, holder))
}
