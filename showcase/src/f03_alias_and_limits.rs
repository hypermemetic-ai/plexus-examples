/// An alias names a type. Its limits are checked before the method runs.
#[plexus::activation(describe)]
pub mod tickets {
    /// A ticket's id: `KILN-2`.
    #[limit(min_len = 3, max_len = 64, pattern = "^[A-Z]+-[0-9]+$")]
    pub type TicketId = String;

    /// 1 is lowest, 5 is drop everything.
    #[limit(min = 1, max = 5)]
    pub type Priority = u8;

    pub struct Tickets;

    impl Tickets {
        pub fn priority(&self, id: TicketId) -> Priority {
            let _ = id;
            3
        }
    }
}
