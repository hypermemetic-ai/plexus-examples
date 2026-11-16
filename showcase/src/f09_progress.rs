/// Progress: what a method reports while the caller waits. Never part of
/// the answer, and not a parameter.
#[plexus::service(describe)]
pub mod export {
    use plexus::Progress;

    pub struct Step {
        pub done: u32,
        pub of: u32,
    }

    pub struct Export;

    impl Export {
        /// Write every ticket to an archive; says how far along it is.
        pub fn archive(&self, under: String, mut progress: Progress<Step>) -> String {
            for done in 1..=4 {
                progress.report(Step { done, of: 4 });
            }
            format!("{under}.tar")
        }
    }
}
