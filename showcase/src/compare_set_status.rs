/// menger's `tickets.set_status`, as it would be written here. For the
/// comparison with today's IR; its event has three of the real one's cases.
#[plexus::activation(describe, roles(editor = ["set_status"]))]
pub mod tickets {
    use plexus::Caller;

    /// A ticket by any form of its address.
    pub type TicketRef = String;
    pub type Message = String;

    pub enum Status {
        Pending,
        Ready,
        Coding,
        InReview,
        Blocked,
        Complete,
        Draft,
        Epic,
        Superseded,
        Canceled,
        Doc,
    }

    pub enum CallerSource {
        App,
        Api,
        Pipeline,
    }

    pub enum TicketsEvent {
        StatusChanged { id: TicketRef, status: Status },
        NotFound(TicketRef),
        Error(String),
    }

    pub struct Tickets;

    impl Tickets {
        /// Set a ticket's status.
        pub fn set_status(
            &self,
            #[target] id: TicketRef,
            message: Option<Message>,
            #[default(false)] restore_children: bool,
            #[default("api")] source: CallerSource,
            status: Status,
            caller: &Caller,
        ) -> impl Iterator<Item = TicketsEvent> {
            let _ = (message, restore_children, source, caller);
            std::iter::once(TicketsEvent::StatusChanged { id, status })
        }
    }
}
