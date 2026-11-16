/// A call that can fail names its error. The error is part of the interface.
#[plexus::activation(describe)]
pub mod tickets {
    pub struct Ticket {
        pub id: String,
    }

    pub enum TicketError {
        NotFound(String),
        /// The title is empty.
        NoTitle,
    }

    pub struct Tickets;

    impl Tickets {
        pub fn open(&self, title: String) -> Result<Ticket, TicketError> {
            if title.is_empty() {
                return Err(TicketError::NoTitle);
            }
            Ok(Ticket { id: "KILN-3".into() })
        }

        /// Nothing comes back; it can still fail.
        pub fn delete(&self, id: String) -> Result<(), TicketError> {
            Err(TicketError::NotFound(id))
        }
    }
}
