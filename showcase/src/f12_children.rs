/// Children: services held under this one. One that is always there, and
/// one per key, with the method that lists the keys.
#[plexus::service(describe)]
pub mod tickets {
    /// Two services of their own, written elsewhere.
    pub struct Commits;
    pub struct Project;

    pub enum NotFound {
        NoSuchProject(String),
    }

    pub struct Tickets {
        commits: Commits,
    }

    impl Tickets {
        /// Always there: `tickets.commits.*`
        #[child]
        pub fn commits(&self) -> &Commits {
            &self.commits
        }

        /// One per name: `tickets.project.<name>.*`
        #[child(list = "projects")]
        pub fn project(&self, name: &str) -> Result<Project, NotFound> {
            if name == "kiln" { Ok(Project) } else { Err(NotFound::NoSuchProject(name.to_owned())) }
        }

        /// The names `project` answers to.
        pub fn projects(&self) -> Vec<String> {
            vec!["kiln".into(), "glaze".into()]
        }
    }
}
