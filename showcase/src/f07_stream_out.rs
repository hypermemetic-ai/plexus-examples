/// A stream out: many items, one at a time, until there are no more.
/// `impl Stream` may wait between items; `impl Iterator` is accepted for
/// one that never does.
#[plexus::activation(describe)]
pub mod tickets {
    use futures::{Stream, StreamExt};

    pub struct Change {
        pub id: String,
        pub status: String,
    }

    pub struct Tickets;

    impl Tickets {
        /// Every change, as it happens.
        pub fn watch(&self, under: String) -> impl Stream<Item = Change> {
            futures::stream::iter(1..=3).map(move |n| Change { id: format!("{under}-{n}"), status: "Ready".into() })
        }

        /// Every id under a ticket: nothing to wait for.
        pub fn ids(&self, under: String) -> impl Iterator<Item = String> {
            (1..=3).map(move |n| format!("{under}-{n}"))
        }
    }
}
