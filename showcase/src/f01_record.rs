/// A record: named fields.
#[plexus::service(describe)]
pub mod tickets {
    /// One ticket.
    pub struct Ticket {
        pub id: String,
        /// What it is about.
        pub title: String,
        pub open: bool,
    }

    pub struct Tickets;

    impl Tickets {
        /// The newest ticket.
        pub fn newest(&self) -> Ticket {
            Ticket { id: "KILN-2".into(), title: "the firing log".into(), open: true }
        }
    }
}
