/// An enum is one of several names. A variant's cases carry nothing, one
/// value, or named fields.
#[plexus::service(describe)]
pub mod tickets {
    pub enum Status {
        Draft,
        Ready,
        Complete,
    }

    pub enum Change {
        /// A ticket was filed.
        Created { id: String, title: String },
        StatusChanged { id: String, status: Status },
        Deleted(String),
        /// Everything went.
        Cleared,
    }

    pub struct Tickets;

    impl Tickets {
        pub fn last_change(&self) -> Change {
            Change::Created { id: "KILN-2".into(), title: "the firing log".into() }
        }
    }
}
