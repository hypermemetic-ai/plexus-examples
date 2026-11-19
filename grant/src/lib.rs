//! A sketch, not a crate of Plexus: what a method is given besides its
//! parameters (one trait, `FromCall`), who a user is to an activation, roles
//! as expressions over a service's methods, grants by method, and a
//! delegation that only narrows. Plexus decides up to the method's
//! invocation; what a user may do to the data is the application's, decided
//! inside the method. Nothing here is served: the kiln below is called by
//! hand, as `#[activation]` would call it.

pub mod call;
pub mod grants;
pub mod path;
pub mod roles;

/// A kiln, as an author would write it, and below it what
/// `#[plexus::activation]` would write for it, by hand.
pub mod kiln {
    use crate::call::{Call, Caller, Ctx, FromCall, Needs, Principal, Refusal, agrees};
    use crate::grants::Grants;
    use crate::path::Path;
    use crate::roles::{Methods, Roles};
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};

    /// The studio's idea of a person: who signed in, and what the studio
    /// keeps about them. The application's type, not plexus's.
    #[derive(Debug, Clone)]
    pub struct User {
        pub principal: Principal,
        pub name: String,
    }

    /// A user is the caller, looked up in the activation's own records.
    impl FromCall<Kiln> for User {
        const NEEDS: Needs = Needs::CALLER;
        fn from_call(call: &Call, kiln: &Kiln) -> Result<User, Refusal> {
            let Caller(p) = Caller::from_call(call, kiln)?;
            kiln.people.borrow().get(&p).cloned().ok_or(Refusal::Unknown { principal: p })
        }
    }

    /// Given to the server at start, read by any method that asks.
    pub struct Limits {
        pub hottest_cone: u8,
    }

    #[derive(Default)]
    pub struct Kiln {
        pub people: RefCell<BTreeMap<Principal, User>>,
        /// The studio's own policy: who keeps which kiln. Plexus never reads it.
        pub keepers: RefCell<BTreeMap<u32, BTreeSet<Principal>>>,
        pub log: RefCell<Vec<String>>,
    }

    /// Why firing did not happen: the method's own typed failure.
    #[derive(Debug, Clone, PartialEq)]
    pub enum FireError {
        /// The caller may fire kilns, but this one is not theirs.
        NotYours { kiln: u32 },
        TooHot { cone: u8 },
    }

    impl Kiln {
        /// The application's access policy for one kiln. Any shape the
        /// application likes: a table, a team, a rule on the user's record.
        fn keeps(&self, user: &User, kiln: u32) -> bool {
            self.keepers.borrow().get(&kiln).is_some_and(|k| k.contains(&user.principal))
        }
    }

    // ---- what the author writes ----
    //
    // #[plexus::activation(
    //     context(User: caller),
    //     roles(viewer = "group:read", firer = "role:viewer || kiln.fire", keeper = "kiln.**"),
    // )]
    impl Kiln {
        // #[group(read)]
        pub fn status(&self) -> String {
            format!("{} firings", self.log.borrow().len())
        }
        // #[group(write)]
        pub fn fire(&self, user: User, limits: Ctx<Limits>, kiln: u32, cone: u8) -> Result<String, FireError> {
            // Plexus let the call in: the caller holds `kiln.fire`. Whether
            // they may fire *this* kiln is the studio's to say, here.
            if !self.keeps(&user, kiln) {
                return Err(FireError::NotYours { kiln });
            }
            if cone > limits.0.hottest_cone {
                return Err(FireError::TooHot { cone });
            }
            self.log.borrow_mut().push(format!("{} fired kiln {kiln} to cone {cone}", user.name));
            Ok(format!("kiln {kiln} firing to cone {cone} for {}", user.name))
        }
        // #[group(read)]
        pub fn whoami(&self, user: Option<User>) -> String {
            user.map(|u| u.name).unwrap_or_else(|| "nobody".into())
        }
    }

    // ---- what the macro writes ----

    // `context(User: caller)` said the user needs a caller; the type agrees,
    // or this does not compile.
    const _: () = assert!(agrees::<Kiln, User>(Needs::CALLER));

    /// The service's methods with their groups: the universe roles are over.
    pub fn methods() -> Methods {
        let mut m = Methods::default();
        // The macro has these as segments already; it never writes the text.
        for (path, group) in [("kiln.status", "read"), ("kiln.fire", "write"), ("kiln.whoami", "read")] {
            m.add(path.parse().expect("the macro's own names"), &[group]);
        }
        m
    }

    /// The roles the attribute declares. An admin's stored definitions are
    /// applied over these at start.
    pub fn roles() -> Roles {
        let mut r = Roles::new(methods());
        r.define("viewer", "group:read").unwrap();
        r.define("firer", "role:viewer || kiln.fire").unwrap();
        r.define("keeper", "kiln.**").unwrap();
        r
    }

    /// What one call answers: a refusal before the method, or the method's
    /// own answer, which may be its typed failure.
    pub type Answer = Result<Result<String, FireError>, Refusal>;

    /// One call, as the served method would run it: decide the method by
    /// grant, take the wire's parameters, fill the context, run. A method
    /// that needs no caller (`whoami` takes `Option<User>`) is let in
    /// anonymously without a grant; `Needs` says which.
    pub fn serve(kiln: &Kiln, grants: &Grants, roles: &Roles, call: &Call, params: &[u32]) -> Answer {
        let path: &Path = &call.path;
        match path.names().as_slice() {
            ["kiln", "status"] => {
                grants.decide(roles, call.principal.as_ref(), path)?;
                Ok(Ok(kiln.status()))
            }
            ["kiln", "fire"] => {
                // The grant first: it needs only the principal and the
                // method, and a caller who may not call learns nothing.
                grants.decide(roles, call.principal.as_ref(), path)?;
                let (target, cone) = (params[0], params[1] as u8);
                let user = User::from_call(call, kiln)?;
                let limits = Ctx::<Limits>::from_call(call, kiln)?;
                Ok(kiln.fire(user, limits, target, cone))
            }
            ["kiln", "whoami"] => Ok(Ok(kiln.whoami(Option::<User>::from_call(call, kiln)?))),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::call::{Call, Extensions, Principal, Refusal};
    use crate::grants::{Grants, Link, Who, authority};
    use crate::kiln::{self, FireError, Kiln, Limits, User};
    use crate::path::{Path, Pattern};
    use crate::roles::{Methods, RoleError, Roles};

    fn path(s: &str) -> Path {
        s.parse().unwrap()
    }
    fn names(paths: Vec<&Path>) -> Vec<String> {
        paths.into_iter().map(Path::to_string).collect()
    }

    const ADA: &str = "key:ada";
    const BEN: &str = "user:idp:ben";

    fn studio() -> (Kiln, Grants, Roles, Extensions) {
        let k = Kiln::default();
        for (p, name) in [(ADA, "Ada"), (BEN, "Ben")] {
            k.people.borrow_mut().insert(Principal(p.into()), User { principal: Principal(p.into()), name: name.into() });
        }
        k.keepers.borrow_mut().insert(3, [Principal(ADA.into())].into());
        let mut g = Grants::default();
        g.grant(Who::One(Principal(ADA.into())), "firer");
        g.grant(Who::Everyone, "viewer");
        let mut ext = Extensions::default();
        ext.insert(Limits { hottest_cone: 10 });
        (k, g, kiln::roles(), ext)
    }

    #[test]
    fn an_anonymous_call_is_told_to_sign_in() {
        let (k, g, r, ext) = studio();
        let call = Call::new(path("kiln.fire")).with_server(ext);
        assert_eq!(kiln::serve(&k, &g, &r, &call, &[3, 6]), Err(Refusal::SignIn));
    }

    #[test]
    fn plexus_lets_the_firer_in_and_the_studio_says_which_kiln() {
        let (k, g, r, ext) = studio();
        let call = Call::new(path("kiln.fire")).by(ADA).with_server(ext);
        assert_eq!(kiln::serve(&k, &g, &r, &call, &[3, 6]), Ok(Ok("kiln 3 firing to cone 6 for Ada".into())));
        // Let in by plexus, refused by the studio's own policy, as the method's answer.
        assert_eq!(kiln::serve(&k, &g, &r, &call, &[4, 6]), Ok(Err(FireError::NotYours { kiln: 4 })));
        assert_eq!(kiln::serve(&k, &g, &r, &call, &[3, 11]), Ok(Err(FireError::TooHot { cone: 11 })));
    }

    #[test]
    fn a_signed_in_person_the_kiln_does_not_know_is_refused_by_name() {
        let (k, g, r, ext) = studio();
        let call = Call::new(path("kiln.whoami")).by("key:stranger").with_server(ext);
        assert_eq!(kiln::serve(&k, &g, &r, &call, &[]), Err(Refusal::Unknown { principal: Principal("key:stranger".into()) }));
    }

    #[test]
    fn a_person_signed_in_through_a_provider_is_a_user_the_same_way() {
        let (k, g, r, ext) = studio();
        let call = Call::new(path("kiln.whoami")).by(BEN).with_server(ext);
        assert_eq!(kiln::serve(&k, &g, &r, &call, &[]), Ok(Ok("Ben".into())));
        // Ben may look, everyone may, but holds no grant to fire.
        let fire = Call::new(path("kiln.fire")).by(BEN).with_server(Extensions::default());
        assert_eq!(kiln::serve(&k, &g, &r, &fire, &[3, 6]), Err(Refusal::Forbidden { grant: path("kiln.fire") }), "the grant is asked before the context is filled");
    }

    #[test]
    fn an_optional_user_lets_anyone_call() {
        let (k, g, r, _) = studio();
        assert_eq!(kiln::serve(&k, &g, &r, &Call::new(path("kiln.whoami")), &[]), Ok(Ok("nobody".into())));
    }

    #[test]
    fn a_value_the_server_was_not_given_is_named() {
        let (k, g, r, _) = studio();
        let call = Call::new(path("kiln.fire")).by(ADA);
        assert!(matches!(kiln::serve(&k, &g, &r, &call, &[3, 6]), Err(Refusal::Missing { what }) if what.ends_with("Limits")));
    }

    fn service() -> Methods {
        let mut m = Methods::default();
        for (p, g) in [
            ("studio.kiln.status", "read"),
            ("studio.kiln.fire", "write"),
            ("studio.kiln.log.list", "read"),
            ("studio.kiln.log.clear", "write"),
            ("studio.glaze.list", "read"),
            ("studio.glaze.mix", "write"),
        ] {
            m.add(path(p), &[g]);
        }
        m
    }

    #[test]
    fn one_method_a_family_a_group_and_their_meet() {
        let mut r = Roles::new(service());
        r.define("one", "studio.kiln.fire").unwrap();
        r.define("kiln-direct", "studio.kiln.*").unwrap();
        r.define("kiln-all", "studio.kiln.**").unwrap();
        r.define("a", "studio.kiln.** && group:read").unwrap();
        r.define("all-reads", "group:read").unwrap();
        let names = |n: &str| names(r.set(n).unwrap().paths(r.methods()));
        assert_eq!(names("one"), ["studio.kiln.fire"]);
        assert_eq!(names("kiln-direct"), ["studio.kiln.status", "studio.kiln.fire"]);
        assert_eq!(names("kiln-all"), ["studio.kiln.status", "studio.kiln.fire", "studio.kiln.log.list", "studio.kiln.log.clear"]);
        assert_eq!(names("a"), ["studio.kiln.status", "studio.kiln.log.list"]);
        assert!(r.set("a").unwrap().within(r.set("all-reads").unwrap()), "role a grants nothing all-reads does not");
        assert!(!r.set("all-reads").unwrap().within(r.set("a").unwrap()));
    }

    #[test]
    fn a_role_that_could_grow_by_itself_is_refused() {
        let mut r = Roles::new(service());
        assert_eq!(r.define("x", "!group:write"), Err(RoleError::Unbounded("!group:write".into())));
        assert!(r.define("x", "studio.** && !group:write").is_ok());
        assert!(r.define("y", "studio.kiln.** || !group:write").is_err());
    }

    #[test]
    fn a_role_that_names_itself_is_refused() {
        let mut r = Roles::new(service());
        r.define("a", "group:read").unwrap();
        r.define("b", "role:a || studio.glaze.mix").unwrap();
        assert!(matches!(r.define("a", "role:b"), Err(RoleError::Cycle(_))));
        assert_eq!(r.set("a").unwrap().paths(r.methods()).len(), 3, "a refused redefinition changes nothing");
    }

    #[test]
    fn a_typo_is_refused_not_granted_nothing() {
        let mut r = Roles::new(service());
        assert_eq!(r.define("a", "studio.klin.*"), Err(RoleError::Matches("studio.klin.*".parse::<Pattern>().unwrap())));
        assert_eq!(r.define("a", "studio.Kiln.*"), Err(RoleError::Parse { at: 7, expected: "a name is lower-case letters, digits, `-` and `_`" }));
        assert!(matches!(r.define("a", "studio.**.fire"), Err(RoleError::Parse { at: 7, .. })));
        assert_eq!(r.define("a", "group:reed"), Err(RoleError::UnknownGroup("reed".into())));
        assert!(matches!(r.define("a", "group:read &&"), Err(RoleError::Parse { .. })));
    }

    #[test]
    fn a_method_added_later_joins_only_the_roles_that_name_it() {
        let mut r = Roles::new(service());
        r.define("reader", "studio.kiln.** && group:read").unwrap();
        r.define("not-writes", "studio.** && !group:write").unwrap();
        r.define("keeper", "studio.kiln.**").unwrap();
        let mut grown = service();
        grown.add(path("studio.kiln.vent"), &["write"]);
        grown.add(path("studio.kiln.temperature"), &["read"]);
        r.remeasure(grown).unwrap();
        assert!(!r.grants("reader", &path("studio.kiln.vent")));
        assert!(r.grants("reader", &path("studio.kiln.temperature")));
        assert!(!r.grants("not-writes", &path("studio.kiln.vent")));
        assert!(r.grants("keeper", &path("studio.kiln.vent")));
    }

    #[test]
    fn which_methods_a_person_may_call_is_one_question() {
        let (_, g, r, _) = studio();
        let reach = |who: &str| -> Vec<String> { g.reach(&r, &Principal(who.into())).iter().map(Path::to_string).collect() };
        assert_eq!(reach(ADA), ["kiln.fire", "kiln.status", "kiln.whoami"]);
        assert_eq!(reach(BEN), ["kiln.status", "kiln.whoami"]);
    }

    #[test]
    fn a_delegation_only_narrows() {
        let mut r = Roles::new(service());
        r.define("kiln", "studio.kiln.**").unwrap();
        r.define("fire", "studio.kiln.fire || studio.glaze.mix").unwrap();
        let p = |s: &str| Principal(s.into());
        let chain = [
            Link { from: p(ADA), to: p("key:studio"), methods: r.set("kiln").unwrap().clone(), until: 100 },
            Link { from: p("key:studio"), to: p("key:glazer"), methods: r.set("fire").unwrap().clone(), until: 50 },
        ];
        let (who, methods, holder) = authority(&chain, 10).unwrap();
        assert_eq!((who, holder), (p(ADA), p("key:glazer")));
        assert_eq!(names(methods.paths(r.methods())), ["studio.kiln.fire"], "glaze.mix was never Ada's to give");
        assert!(authority(&chain, 60).is_none(), "the shortest expiry is the chain's");
        let broken = [chain[1].clone(), chain[0].clone()];
        assert!(authority(&broken, 10).is_none());
    }
}
