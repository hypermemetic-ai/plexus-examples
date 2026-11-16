/// A case is written as its kebab name unless it says otherwise:
/// `#[wire("In Review")]` keeps a spelling the wire already has.
#[plexus::activation(describe)]
pub mod tickets {
    pub enum Status {
        Draft,
        #[wire("In Review")]
        InReview,
        Complete,
    }

    pub enum Change {
        #[wire("Moved")]
        StatusChanged { id: String, status: Status },
        #[wire("Gone")]
        Deleted(String),
        #[wire("All gone")]
        Cleared,
    }

    pub struct Tickets;

    impl Tickets {
        pub fn set_status(&self, id: String, status: Status) -> Change {
            Change::StatusChanged { id, status }
        }
    }
}
