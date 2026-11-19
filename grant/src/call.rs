//! What a call carries besides its parameters, and the one trait by which a
//! method's other arguments are filled from it.

use crate::path::Path;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

/// Who someone is: `key:<64 hex>` (a key proved in TLS, or a passkey) or
/// `user:<provider>:<id>` (a sign-in through a provider).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Principal(pub String);

/// One call, as the server sees it before the method runs.
pub struct Call {
    /// The method's served path: `studio.kiln.fire`, parsed where the
    /// call came in.
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

/// What the macro checks, at compile time, of each argument it fills: that
/// what the IR says the argument needs is what the type says. The IR is
/// written from the source alone, so the source says it once
/// (`context(User: caller)`), and this keeps the two the same.
pub const fn agrees<A, T: FromCall<A>>(ir: Needs) -> bool {
    T::NEEDS.caller == ir.caller
}
