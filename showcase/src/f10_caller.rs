/// The user: `&Caller` is who is calling. A method that takes one needs a
/// signed-in caller; `Option<&Caller>` lets anyone in. Neither is sent as
/// a parameter.
#[plexus::activation(describe)]
pub mod session {
    use plexus::Caller;

    pub struct Session;

    impl Session {
        /// Refused unless signed in.
        pub fn whoami(&self, caller: &Caller) -> String {
            caller.principal().to_owned()
        }

        /// Anyone may ask.
        pub fn greeting(&self, caller: Option<&Caller>) -> String {
            match caller {
                Some(c) => format!("hello, {}", c.principal()),
                None => "hello, stranger".into(),
            }
        }
    }
}
