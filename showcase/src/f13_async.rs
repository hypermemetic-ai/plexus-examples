/// An `async fn` may wait: for a disk, for another activation. The IR does not
/// say which methods wait, because over a wire every call is awaited. WIT
/// does: a caller in the same process gets a promise, or a plain value.
#[plexus::activation(describe)]
pub mod tickets {
    pub struct Ticket {
        pub id: String,
        pub title: String,
    }

    pub struct Tickets;

    impl Tickets {
        /// Waits for the store.
        pub async fn get(&self, id: String) -> Option<Ticket> {
            plexus::yield_now().await;
            Some(Ticket { id, title: "the firing log".into() })
        }

        /// Answers at once.
        pub fn count(&self) -> u32 {
            1
        }
    }
}
