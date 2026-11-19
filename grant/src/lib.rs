//! A sketch, not a crate of Plexus: who is calling, and what they may call.
//!
//! An activation is a function from a context to the methods it may be
//! called with. Its context is a struct: some fields the server fills from
//! the call, the rest the caller sends, as the keys of the activation's
//! segment of the path. Opening runs once, and what it answers, the
//! application's own value, is what every method is given. A child per key
//! is the same one level down: `studio[tenant=acme].kiln[3].fire`.
//!
//! Plexus decides up to the method: whether this caller holds a role with
//! `studio.kiln.fire` in it. Whether they may have this studio, or this
//! kiln, is the application's, decided where each is opened.
//!
//! Nothing here is served. The studio below is called by hand, as
//! `#[activation]` would call it; what the macros would write is marked.

pub mod call;
pub mod grants;
pub mod path;
pub mod roles;

/// A studio and the kilns in it, as an author would write them, and below
/// that what the macros would write, by hand.
pub mod studio {
    use crate::call::{Call, Caller, Connection, Context, Ctx, FromCall, Needs, Principal, Refusal};
    use crate::grants::Grants;
    use crate::path::{Keys, Path};
    use crate::roles::{Methods, Roles};
    use std::cell::{Cell, RefCell};
    use std::collections::{BTreeMap, BTreeSet};
    use std::rc::Rc;

    /// The studio's idea of a person: who signed in, and what the studio
    /// keeps about them. The application's type, not plexus's.
    #[derive(Debug, Clone)]
    pub struct User {
        pub principal: Principal,
        pub name: String,
    }

    /// A user is the caller, looked up in the studio's own records.
    impl FromCall<Studio> for User {
        const NEEDS: Needs = Needs::CALLER;
        fn from_call(call: &Call, studio: &Studio) -> Result<User, Refusal> {
            let Caller(p) = Caller::from_call(call, studio)?;
            studio.people.borrow().get(&p).cloned().ok_or(Refusal::Unknown { principal: p })
        }
    }

    /// Given to the server at start.
    pub struct Limits {
        pub hottest_cone: u8,
    }

    // ---- what the author writes ----

    /// What the studio is opened with.
    // #[derive(plexus::Context)]
    pub struct StudioCx {
        // #[from(caller)]
        pub user: User,
        // #[from(server)]
        pub limits: Rc<Limits>,
        /// No attribute: the caller sends it. `studio[tenant=acme]`.
        pub tenant: String,
    }

    /// What opening the studio gives: proof that this user is a member of
    /// this tenant. Every method of the studio takes it, so none can be
    /// reached without it and none checks membership again.
    pub struct Member {
        pub user: User,
        pub tenant: String,
        pub limits: Rc<Limits>,
    }

    /// What opening a kiln gives: proof that this member keeps it.
    pub struct KilnOf {
        pub number: u32,
        pub keeper: String,
        pub hottest_cone: u8,
    }

    /// The application's own no, typed: at an opening or in a method.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Denied {
        NotAMember { tenant: String },
        NotYours { kiln: u32 },
        TooHot { cone: u8 },
    }

    #[derive(Default)]
    pub struct Studio {
        pub people: RefCell<BTreeMap<Principal, User>>,
        /// The studio's own policy. Plexus never reads either table.
        pub members: RefCell<BTreeMap<String, BTreeSet<Principal>>>,
        pub keepers: RefCell<BTreeMap<(String, u32), BTreeSet<Principal>>>,
        /// Counted up whenever the policy changes, so what was opened
        /// under the old one is opened again.
        pub policy: Cell<u64>,
        pub kiln: Kiln,
    }

    /// The studio's policy, as two questions. Any shape would do.
    impl Studio {
        fn is_member(&self, user: &User, tenant: &str) -> bool {
            self.members.borrow().get(tenant).is_some_and(|m| m.contains(&user.principal))
        }
        fn keeps(&self, me: &Member, kiln: u32) -> bool {
            self.keepers.borrow().get(&(me.tenant.clone(), kiln)).is_some_and(|k| k.contains(&me.user.principal))
        }
    }

    #[derive(Default)]
    pub struct Kiln {
        pub log: RefCell<Vec<String>>,
    }

    // #[plexus::activation(
    //     context = StudioCx,
    //     roles(viewer = "group:read", firer = "role:viewer || studio.kiln.fire", keeper = "studio.**"),
    // )]
    impl Studio {
        /// The door: the context in, the proof out, or the studio's no.
        // #[open]
        pub fn open(&self, cx: StudioCx) -> Result<Member, Denied> {
            if !self.is_member(&cx.user, &cx.tenant) {
                return Err(Denied::NotAMember { tenant: cx.tenant });
            }
            Ok(Member { user: cx.user, tenant: cx.tenant, limits: cx.limits })
        }

        // #[group(read)]
        pub fn status(&self, me: &Member) -> String {
            format!("{}: {} firings, seen by {}", me.tenant, self.kiln.log.borrow().len(), me.user.name)
        }

        /// A kiln, by its number: the same shape one level down.
        // #[child]
        pub fn kiln(&self, me: &Member, number: u32) -> Result<KilnOf, Denied> {
            if !self.keeps(me, number) {
                return Err(Denied::NotYours { kiln: number });
            }
            Ok(KilnOf { number, keeper: me.user.name.clone(), hottest_cone: me.limits.hottest_cone })
        }
    }

    // #[plexus::activation]
    impl Kiln {
        /// No kiln number and no user: `at` is both, already decided.
        // #[group(write)]
        pub fn fire(&self, at: &KilnOf, cone: u8) -> Result<String, Denied> {
            if cone > at.hottest_cone {
                return Err(Denied::TooHot { cone });
            }
            self.log.borrow_mut().push(format!("{} fired kiln {} to cone {cone}", at.keeper, at.number));
            Ok(format!("kiln {} firing to cone {cone} for {}", at.number, at.keeper))
        }

        // #[group(read)]
        pub fn log(&self, at: &KilnOf) -> String {
            format!("kiln {}: {}", at.number, self.log.borrow().join("; "))
        }
    }

    // ---- what #[derive(plexus::Context)] writes ----

    impl Context<Studio> for StudioCx {
        const SENT: &'static [&'static str] = &["tenant"];
        const NEEDS: Needs = Needs::CALLER;
        fn fill(call: &Call, studio: &Studio, at: &Path, keys: &Keys) -> Result<StudioCx, Refusal> {
            Ok(StudioCx {
                user: User::from_call(call, studio)?,
                limits: Ctx::<Limits>::from_call(call, studio)?.0,
                tenant: keys.named("tenant").ok_or(Refusal::BadKey { at: at.clone(), why: "the studio is opened with `tenant`" })?.as_str().into(),
            })
        }
    }

    // ---- what #[plexus::activation] writes ----

    /// The service's methods with their groups: what roles are over. No
    /// keys: a method is the same method on every kiln.
    pub fn methods() -> Methods {
        let mut m = Methods::default();
        for (path, group) in [("studio.status", "read"), ("studio.kiln.fire", "write"), ("studio.kiln.log", "read")] {
            m.add(path.parse().expect("the macro's own names"), &[group]);
        }
        m
    }

    /// The roles the attribute declares. An admin's stored definitions are
    /// applied over these at start.
    pub fn roles() -> Roles {
        let mut r = Roles::new(methods());
        r.define("viewer", "group:read").unwrap();
        r.define("firer", "role:viewer || studio.kiln.fire").unwrap();
        r.define("keeper", "studio.**").unwrap();
        r
    }

    /// What one call answers: plexus's refusal before anything of the
    /// application ran, or the application's own answer.
    pub type Answer = Result<Result<String, Denied>, Refusal>;

    enum Stop {
        Refused(Refusal),
        Denied(Denied),
    }
    impl From<Refusal> for Stop {
        fn from(r: Refusal) -> Stop {
            Stop::Refused(r)
        }
    }
    impl From<Denied> for Stop {
        fn from(d: Denied) -> Stop {
            Stop::Denied(d)
        }
    }

    /// One call, as it would be served.
    pub fn serve(studio: &Studio, grants: &Grants, roles: &Roles, conn: &Connection, call: &Call, params: &[u32]) -> Answer {
        match run(studio, grants, roles, conn, call, params) {
            Ok(v) => Ok(Ok(v)),
            Err(Stop::Denied(d)) => Ok(Err(d)),
            Err(Stop::Refused(r)) => Err(r),
        }
    }

    fn run(studio: &Studio, grants: &Grants, roles: &Roles, conn: &Connection, call: &Call, params: &[u32]) -> Result<String, Stop> {
        let path = &call.path;
        // 1. Plexus: may this caller call this method at all? By the path's
        //    shape alone, before anything is opened or looked up.
        grants.decide(roles, call.principal.as_ref(), &path.shape())?;
        let who = call.principal.as_ref().expect("decided for someone");
        // What an opening's answer rests on: the grants and the studio's policy.
        let generation = grants.generation() + studio.policy.get();
        let segs = path.segments();

        // 2. The studio, opened with its context: once a connection, then remembered.
        let me: Rc<Member> = conn.opened(who, path.prefix(1), generation, || -> Result<Member, Stop> {
            Ok(studio.open(StudioCx::fill(call, studio, &path.prefix(1), &segs[0].keys)?)?)
        })?;

        match path.names().as_slice() {
            ["studio", "status"] => Ok(studio.status(&me)),
            ["studio", "kiln", method] => {
                // 3. The kiln, opened with its key through the studio already open.
                let at: Rc<KilnOf> = conn.opened(who, path.prefix(2), generation, || -> Result<KilnOf, Stop> {
                    let bad = |why| Refusal::BadKey { at: path.prefix(2), why };
                    let number = segs[1].keys.one().ok_or(bad("a kiln is opened with its number"))?.as_str().parse().map_err(|_| bad("a kiln's number is a number"))?;
                    Ok(studio.kiln(&me, number)?)
                })?;
                // 4. The method, given the proof.
                match *method {
                    "fire" => Ok(studio.kiln.fire(&at, params[0] as u8)?),
                    "log" => Ok(studio.kiln.log(&at)),
                    _ => unreachable!("decided against the methods"),
                }
            }
            _ => unreachable!("decided against the methods"),
        }
    }

    /// What `plexus-gen` would write for a caller: a handle is an address.
    /// Opening sends nothing; it builds the path the call will carry.
    pub mod client {
        use crate::path::{Name, Path, Value};

        fn name(s: &str) -> Name {
            Name::new(s).expect("a generated name")
        }

        pub struct Studio(Path);
        pub struct Kiln(Path);

        /// `open({ tenant })`: the context's sent fields.
        pub fn open(tenant: &str) -> Result<Studio, crate::path::PathError> {
            Ok(Studio(Path::root(name("studio")).with(name("tenant"), Value::new(tenant)?)))
        }

        impl Studio {
            pub fn status(&self) -> Path {
                self.0.clone().then(name("status"))
            }
            pub fn kiln(&self, number: u32) -> Kiln {
                Kiln(self.0.clone().then(name("kiln")).at(Value::new(&number.to_string()).expect("a number is a key")))
            }
        }

        impl Kiln {
            pub fn fire(&self) -> Path {
                self.0.clone().then(name("fire"))
            }
            pub fn log(&self) -> Path {
                self.0.clone().then(name("log"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::call::{Call, Connection, Context, Extensions, FromCall, Principal, Refusal};
    use crate::grants::{Grants, Link, Who, authority};
    use crate::path::{Path, Pattern};
    use crate::roles::{Methods, RoleError, Roles};
    use crate::studio::{self, Denied, Limits, Studio, StudioCx, User, client};

    fn path(s: &str) -> Path {
        s.parse().unwrap()
    }
    fn names(paths: Vec<&Path>) -> Vec<String> {
        paths.into_iter().map(Path::to_string).collect()
    }
    fn p(s: &str) -> Principal {
        Principal(s.into())
    }

    const ADA: &str = "key:ada";
    const BEN: &str = "user:idp:ben";

    /// Ada and Ben are members of acme. Ada keeps kiln 3 and may fire;
    /// everyone may look.
    struct World {
        studio: Studio,
        grants: Grants,
        roles: Roles,
    }

    fn world() -> World {
        let studio = Studio::default();
        for (who, name) in [(ADA, "Ada"), (BEN, "Ben")] {
            studio.people.borrow_mut().insert(p(who), User { principal: p(who), name: name.into() });
        }
        studio.members.borrow_mut().insert("acme".into(), [p(ADA), p(BEN)].into());
        studio.keepers.borrow_mut().insert(("acme".into(), 3), [p(ADA)].into());
        let mut grants = Grants::default();
        grants.grant(Who::One(p(ADA)), "firer");
        grants.grant(Who::Everyone, "viewer");
        World { studio, grants, roles: studio::roles() }
    }

    fn limits() -> Extensions {
        let mut ext = Extensions::default();
        ext.insert(Limits { hottest_cone: 10 });
        ext
    }

    impl World {
        fn call(&self, conn: &Connection, who: Option<&str>, at: &str, params: &[u32]) -> studio::Answer {
            let mut call = Call::new(path(at)).with_server(limits());
            if let Some(who) = who {
                call = call.by(who);
            }
            studio::serve(&self.studio, &self.grants, &self.roles, conn, &call, params)
        }
    }

    const FIRE_3: &str = "studio[tenant=acme].kiln[3].fire";

    #[test]
    fn an_anonymous_call_is_told_to_sign_in() {
        let w = world();
        assert_eq!(w.call(&Connection::default(), None, FIRE_3, &[6]), Err(Refusal::SignIn));
    }

    #[test]
    fn the_address_opens_the_studio_then_the_kiln_then_fires() {
        let w = world();
        let conn = Connection::default();
        assert_eq!(w.call(&conn, Some(ADA), FIRE_3, &[6]), Ok(Ok("kiln 3 firing to cone 6 for Ada".into())));
        assert_eq!(w.call(&conn, Some(ADA), "studio[tenant=acme].kiln[3].log", &[]), Ok(Ok("kiln 3: Ada fired kiln 3 to cone 6".into())));
        assert_eq!(w.call(&conn, Some(BEN), "studio[tenant=acme].status", &[]), Ok(Ok("acme: 1 firings, seen by Ben".into())));
    }

    #[test]
    fn plexus_refuses_by_method_before_anything_is_opened() {
        let w = world();
        // Ben is a member and may look, but holds no role with `studio.kiln.fire`.
        let conn = Connection::default();
        assert_eq!(w.call(&conn, Some(BEN), FIRE_3, &[6]), Err(Refusal::Forbidden { grant: path("studio.kiln.fire") }));
        assert_eq!(conn.ran(), 0, "no opening ran, and no record was read");
    }

    #[test]
    fn the_application_says_no_where_each_level_is_opened() {
        let w = world();
        let conn = Connection::default();
        // Let in by plexus each time; the studio's own answers.
        assert_eq!(w.call(&conn, Some(ADA), "studio[tenant=other].kiln[3].fire", &[6]), Ok(Err(Denied::NotAMember { tenant: "other".into() })));
        assert_eq!(w.call(&conn, Some(ADA), "studio[tenant=acme].kiln[4].fire", &[6]), Ok(Err(Denied::NotYours { kiln: 4 })));
        assert_eq!(w.call(&conn, Some(ADA), FIRE_3, &[11]), Ok(Err(Denied::TooHot { cone: 11 })));
    }

    #[test]
    fn an_opening_runs_once_a_connection_and_the_wire_carries_no_state() {
        let w = world();
        let conn = Connection::default();
        for _ in 0..3 {
            w.call(&conn, Some(ADA), FIRE_3, &[6]).unwrap().unwrap();
        }
        w.call(&conn, Some(ADA), "studio[tenant=acme].kiln[3].log", &[]).unwrap().unwrap();
        assert_eq!(conn.ran(), 2, "four calls: the studio opened once, the kiln once");
        // A connection that knows nothing, as after a reconnect: the same call, the same answer.
        let again = Connection::default();
        assert_eq!(w.call(&again, Some(ADA), FIRE_3, &[6]), Ok(Ok("kiln 3 firing to cone 6 for Ada".into())));
        assert_eq!(again.ran(), 2);
        // A refusal is not remembered.
        assert!(w.call(&conn, Some(ADA), "studio[tenant=acme].kiln[4].fire", &[6]).unwrap().is_err());
        assert!(w.call(&conn, Some(ADA), "studio[tenant=acme].kiln[4].fire", &[6]).unwrap().is_err());
        assert_eq!(conn.ran(), 4, "kiln 4 was asked for twice, and opened neither time");
    }

    #[test]
    fn what_was_opened_is_opened_again_when_the_policy_or_the_grants_change() {
        let mut w = world();
        let conn = Connection::default();
        w.call(&conn, Some(ADA), FIRE_3, &[6]).unwrap().unwrap();
        // The studio takes kiln 3 from Ada, and says its policy changed.
        w.studio.keepers.borrow_mut().clear();
        w.studio.policy.set(w.studio.policy.get() + 1);
        assert_eq!(w.call(&conn, Some(ADA), FIRE_3, &[6]), Ok(Err(Denied::NotYours { kiln: 3 })), "on the connection that had it open");
        // An admin takes the role: plexus refuses at the next call.
        w.grants.revoke(Who::One(p(ADA)), "firer");
        assert_eq!(w.call(&conn, Some(ADA), FIRE_3, &[6]), Err(Refusal::Forbidden { grant: path("studio.kiln.fire") }));
    }

    #[test]
    fn a_level_opened_with_the_wrong_keys_is_refused_by_its_address() {
        let w = world();
        let conn = Connection::default();
        assert_eq!(
            w.call(&conn, Some(ADA), "studio.kiln[3].fire", &[6]),
            Err(Refusal::BadKey { at: path("studio"), why: "the studio is opened with `tenant`" })
        );
        assert_eq!(
            w.call(&conn, Some(ADA), "studio[tenant=acme].kiln[three].fire", &[6]),
            Err(Refusal::BadKey { at: path("studio[tenant=acme].kiln[three]"), why: "a kiln's number is a number" })
        );
        assert_eq!(StudioCx::SENT, ["tenant"], "what the IR says a caller sends to open the studio");
    }

    #[test]
    fn a_handle_is_an_address_built_without_a_call() {
        let studio = client::open("acme").unwrap();
        assert_eq!(studio.kiln(3).fire(), path(FIRE_3));
        assert_eq!(studio.status().to_string(), "studio[tenant=acme].status");
        assert!(client::open("acme].kiln[4").is_err(), "a key cannot reach another level");
        // The handle's path is the call.
        let w = world();
        let call = Call::new(studio.kiln(3).fire()).by(ADA).with_server(limits());
        assert_eq!(studio::serve(&w.studio, &w.grants, &w.roles, &Connection::default(), &call, &[6]), Ok(Ok("kiln 3 firing to cone 6 for Ada".into())));
    }

    #[test]
    fn the_context_fields_are_filled_each_from_its_place() {
        let w = world();
        let at = path("studio[tenant=acme]");
        let keys = &at.segments()[0].keys;
        // A stranger who signed in is not a user of this studio.
        let stranger = Call::new(at.clone()).by("key:stranger").with_server(limits());
        assert!(matches!(StudioCx::fill(&stranger, &w.studio, &at, keys), Err(Refusal::Unknown { principal }) if principal == p("key:stranger")));
        // A server never given its limits says which value is missing.
        let bare = Call::new(at.clone()).by(ADA);
        assert!(matches!(StudioCx::fill(&bare, &w.studio, &at, keys), Err(Refusal::Missing { what }) if what.ends_with("Limits")));
        // A field may be optional: anonymous is `None`, a stranger is still refused.
        assert!(Option::<User>::from_call(&Call::new(at.clone()), &w.studio).unwrap().is_none());
        assert!(Option::<User>::from_call(&stranger, &w.studio).is_err());
    }

    fn service() -> Methods {
        let mut m = Methods::default();
        for (path_, g) in [
            ("studio.kiln.status", "read"),
            ("studio.kiln.fire", "write"),
            ("studio.kiln.log.list", "read"),
            ("studio.kiln.log.clear", "write"),
            ("studio.glaze.list", "read"),
            ("studio.glaze.mix", "write"),
        ] {
            m.add(path(path_), &[g]);
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
        assert!(r.grants("one", &path("studio[tenant=acme].kiln[3].fire")), "a role holds the method on every kiln");
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
        assert!(r.define("a", "studio.kiln[3].fire").is_err(), "a role cannot name one kiln: that is the application's");
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
        let w = world();
        let reach = |who: &str| -> Vec<String> { w.grants.reach(&w.roles, &p(who)).iter().map(Path::to_string).collect() };
        assert_eq!(reach(ADA), ["studio.kiln.fire", "studio.kiln.log", "studio.status"]);
        assert_eq!(reach(BEN), ["studio.kiln.log", "studio.status"]);
    }

    #[test]
    fn a_delegation_only_narrows() {
        let mut r = Roles::new(service());
        r.define("kiln", "studio.kiln.**").unwrap();
        r.define("fire", "studio.kiln.fire || studio.glaze.mix").unwrap();
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
