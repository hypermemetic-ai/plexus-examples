/// A potter's studio: the kiln in this process, and the glaze activation in
/// another, started as a child and held across its stdin and stdout.
#[plexus::activation]
pub mod studio {
    use kiln::kiln::Kiln;
    use plexus::{Activation, Remote};
    use std::path::PathBuf;

    pub struct Studio {
        kiln: Kiln,
        glaze: Remote,
    }

    impl Studio {
        /// A studio with a cold kiln, and the `glaze` binary started as its
        /// child: the one `STUDIO_GLAZE` names, else the one beside this
        /// program.
        pub fn new() -> std::io::Result<Studio> {
            let glaze = match std::env::var_os("STUDIO_GLAZE") {
                Some(path) => PathBuf::from(path),
                None => std::env::current_exe()?.with_file_name("glaze"),
            };
            Ok(Studio { kiln: Kiln::default(), glaze: Remote::child(tokio::process::Command::new(glaze), "glaze")? })
        }

        /// What the studio holds.
        pub fn about(&self) -> String {
            "a kiln, here; glaze recipes, in another process".into()
        }

        /// How many firings the kiln has logged.
        pub fn fired(&self) -> u32 {
            self.kiln.log().len() as u32
        }

        /// The kiln, in this process.
        #[child]
        pub fn kiln(&self) -> &Kiln {
            &self.kiln
        }

        /// Glaze recipes, across a channel: another process.
        #[child]
        pub fn glaze(&self) -> Box<dyn Activation> {
            Box::new(self.glaze.clone())
        }
    }
}
