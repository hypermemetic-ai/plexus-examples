/// Grants and roles. Every method is its own grant. A role is a list of
/// grants, and may include another role. `#[target]` marks the parameter
/// that names what the grant must be held on.
#[plexus::activation(describe, roles(viewer = ["get", "list"], editor = [viewer, "set_status"], admin = [editor, "delete"]))]
pub mod tickets {
    use plexus::Caller;

    pub struct Tickets;

    impl Tickets {
        /// A viewer of this ticket may read it.
        pub fn get(&self, #[target] id: String, caller: &Caller) -> String {
            format!("{id}, read by {}", caller.principal())
        }

        /// Answers only what the caller may see.
        pub fn list(&self, caller: &Caller) -> Vec<String> {
            let _ = caller;
            vec![]
        }

        /// An editor of this ticket may move it.
        pub fn set_status(&self, #[target] id: String, status: String, caller: &Caller) -> String {
            format!("{id} is {status}, by {}", caller.principal())
        }

        pub fn delete(&self, #[target] id: String, caller: &Caller) -> bool {
            let _ = (id, caller);
            true
        }
    }
}
