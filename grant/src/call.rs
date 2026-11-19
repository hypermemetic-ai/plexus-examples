//! What a call carries besides its parameters, and the one trait by which a
//! method's other arguments are filled from it.

use crate::path::{Keys, Path};
use std::cell::{Cell, RefCell};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

/// Who someone is: `key:<64 hex>` (a key proved in TLS, or a passkey) or
/// `user:<provider>:<id>` (a sign-in through a provider).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Principal(pub String);

/// One call, as the server sees it before the method runs.
pub struct Call {
    /// The call's address: `studio[tenant=acme].kiln[3].fire`, parsed where
    /// the call came in.
    pub path: Path,
    /// The key the far end of this connection proved in TLS. A browser's
    /// connection has none.
    pub peer: Option<Principal>,
    /// Who the call is for. Over mTLS it is the peer; over a signed-in
    /// browser socket it is the session's; through a parent it is what the
    /// on-behalf-of rule says (not decided).
    pub principal: Option<Principal>,
    /// The principals the call came through, outermost first.
    pub via: Vec<Principal>,
    /// Values the server was given at start, one per type (`Ctx<T>`).
    pub(crate) server: Rc<Extensions>,
    /// Values attached to this connection after it opened (`Session<T>`).
    pub(crate) session: Rc<Extensions>,
}

/// A map of one value per type.
#[derive(Default)]
pub struct Extensions(HashMap<TypeId, Rc<dyn Any>>);

impl Extensions {
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.0.insert(TypeId::of::<T>(), Rc::new(value));
    }
    pub fn get<T: 'static>(&self) -> Option<Rc<T>> {
        self.0.get(&TypeId::of::<T>()).cloned().and_then(|v| v.downcast().ok())
    }
}

/// Why a call is refused before its method runs. Typed on the wire.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// The method needs someone, and the call is anonymous.
    SignIn,
    /// Someone called and may not: the grant the call needed.
    Forbidden { grant: Path },
    /// A value the method reads from the server was never given it.
    Missing { what: &'static str },
    /// The principal is not known to this activation.
    Unknown { principal: Principal },
    /// A level of the path was opened with a key it cannot read, or none.
    BadKey { at: Path, why: &'static str },
}

/// What an argument asks of a call. It is in the IR, so a client knows,
/// before calling, that it must sign in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Needs {
    pub caller: bool,
}

impl Needs {
    pub const NOTHING: Needs = Needs { caller: false };
    pub const CALLER: Needs = Needs { caller: true };
}

/// A method's argument filled from the call, not sent by the caller.
/// `A` is the activation, so an argument may look itself up in its state.
pub trait FromCall<A>: Sized {
    const NEEDS: Needs;
    fn from_call(call: &Call, activation: &A) -> Result<Self, Refusal>;
}

/// Who is calling: refused to an anonymous call.
#[derive(Debug, Clone)]
pub struct Caller(pub Principal);

impl Caller {
    pub fn principal(&self) -> &Principal {
        &self.0
    }
}

impl<A> FromCall<A> for Caller {
    const NEEDS: Needs = Needs::CALLER;
    fn from_call(call: &Call, _: &A) -> Result<Self, Refusal> {
        call.principal.clone().map(Caller).ok_or(Refusal::SignIn)
    }
}

/// Any argument made optional: an anonymous call gets `None` where the
/// argument would have refused it for want of a caller. Every other refusal
/// still refuses.
impl<A, T: FromCall<A>> FromCall<A> for Option<T> {
    const NEEDS: Needs = Needs::NOTHING;
    fn from_call(call: &Call, activation: &A) -> Result<Self, Refusal> {
        match T::from_call(call, activation) {
            Ok(v) => Ok(Some(v)),
            Err(Refusal::SignIn) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// A value the server was given when it started: a configuration, a
/// client of another service, a clock. Shared by every call.
pub struct Ctx<T>(pub Rc<T>);

impl<A, T: 'static> FromCall<A> for Ctx<T> {
    const NEEDS: Needs = Needs::NOTHING;
    fn from_call(call: &Call, _: &A) -> Result<Self, Refusal> {
        call.server.get::<T>().map(Ctx).ok_or(Refusal::Missing { what: std::any::type_name::<T>() })
    }
}

/// A value attached to this connection: set once (at sign-in, say) and read
/// by every call on it after.
pub struct Session<T>(pub Rc<T>);

impl<A, T: 'static> FromCall<A> for Session<T> {
    const NEEDS: Needs = Needs::NOTHING;
    fn from_call(call: &Call, _: &A) -> Result<Self, Refusal> {
        call.session.get::<T>().map(Session).ok_or(Refusal::Missing { what: std::any::type_name::<T>() })
    }
}

/// The whole call, untyped: for what no argument type says.
impl<A> FromCall<A> for Rc<CallView> {
    const NEEDS: Needs = Needs::NOTHING;
    fn from_call(call: &Call, _: &A) -> Result<Self, Refusal> {
        Ok(Rc::new(CallView { path: call.path.clone(), peer: call.peer.clone(), principal: call.principal.clone(), via: call.via.clone() }))
    }
}

/// What a method sees of the call when it asks for all of it.
#[derive(Debug, Clone)]
pub struct CallView {
    pub path: Path,
    pub peer: Option<Principal>,
    pub principal: Option<Principal>,
    pub via: Vec<Principal>,
}

impl Call {
    pub fn new(path: Path) -> Call {
        Call { path, peer: None, principal: None, via: Vec::new(), server: Rc::default(), session: Rc::default() }
    }
    pub fn by(mut self, p: &str) -> Call {
        self.principal = Some(Principal(p.into()));
        self
    }
    pub fn with_server(mut self, ext: Extensions) -> Call {
        self.server = Rc::new(ext);
        self
    }
}

/// An activation's context: what it is opened with. A struct, each field
/// said to come from the call (`#[from(caller)]`, `#[from(server)]`,
/// `#[from(session)]`, each a `FromCall`) or, with no attribute, sent by the
/// caller as a key of the activation's segment. `#[derive(plexus::Context)]`
/// writes this from the struct, and the fields sent are in the IR.
pub trait Context<A>: Sized {
    /// The fields the caller sends, by name: the keys of the segment.
    const SENT: &'static [&'static str];
    const NEEDS: Needs;
    fn fill(call: &Call, activation: &A, at: &Path, keys: &Keys) -> Result<Self, Refusal>;
}

/// One connection's memory of what it has opened. Opening a level is a
/// function of who is calling and the level's address, so its answer is
/// kept and the function is not run again, until anything that could change
/// the answer changes: `generation` is that count. Nothing here is state a
/// caller depends on: a new connection, or a dropped entry, opens again and
/// gets the same answer.
#[derive(Default)]
pub struct Connection {
    opened: RefCell<HashMap<(Principal, Path), (u64, Rc<dyn Any>)>>,
    ran: Cell<u32>,
}

impl Connection {
    /// How many openings this connection has had to run, not remember.
    pub fn ran(&self) -> u32 {
        self.ran.get()
    }

    pub fn opened<T: 'static, E>(&self, who: &Principal, at: Path, generation: u64, open: impl FnOnce() -> Result<T, E>) -> Result<Rc<T>, E> {
        let key = (who.clone(), at);
        if let Some((kept_at, kept)) = self.opened.borrow().get(&key) {
            if *kept_at == generation {
                if let Ok(v) = kept.clone().downcast::<T>() {
                    return Ok(v);
                }
            }
        }
        // A refusal is never kept: the next call asks again.
        self.ran.set(self.ran.get() + 1);
        let v = Rc::new(open()?);
        self.opened.borrow_mut().insert(key, (generation, v.clone()));
        Ok(v)
    }
}
