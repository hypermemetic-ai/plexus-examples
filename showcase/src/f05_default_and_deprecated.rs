/// A default is what an absent value is taken as. A deprecated parameter
/// still works, and says what to use.
#[plexus::service(describe)]
pub mod tickets {
    pub struct Page {
        pub ids: Vec<String>,
        #[default(false)]
        pub more: bool,
    }

    pub struct Tickets;

    impl Tickets {
        pub fn list(
            &self,
            #[default(50)] limit: u32,
            #[default("")] after: String,
            #[deprecated(since = "0.3", note = "use `limit`")] count: Option<u32>,
        ) -> Page {
            let _ = (limit, after, count);
            Page { ids: vec![], more: false }
        }
    }
}
