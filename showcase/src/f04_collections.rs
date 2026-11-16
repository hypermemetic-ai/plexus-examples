/// Lists, options, tuples, string-keyed maps, bytes, and any JSON.
#[plexus::activation(describe)]
pub mod tickets {
    use plexus::{Bytes, Json};
    use std::collections::BTreeMap;

    pub struct Ticket {
        pub labels: Vec<String>,
        pub parent: Option<String>,
        /// Row and column on the board.
        pub cell: (u32, u32),
        pub counts: BTreeMap<String, u32>,
        pub avatar: Bytes,
        /// Whatever an integration stored.
        pub extra: Json,
    }

    pub struct Tickets;

    impl Tickets {
        pub fn get(&self, id: String) -> Option<Ticket> {
            let _ = id;
            None
        }
    }
}
