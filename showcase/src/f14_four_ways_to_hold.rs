/// The four ways a service holds another, by what is known when the
/// parent is compiled.
#[plexus::service(describe)]
pub mod studio {
    use plexus::Service;

    /// A service written in this crate.
    pub struct Kiln;

    /// An interface: any service with these methods fits.
    pub trait Glazes {
        fn recipes(&self) -> Vec<String>;
    }

    pub enum NotFound {
        NoSuch(String),
    }

    pub struct Studio {
        kiln: Kiln,
    }

    impl Studio {
        /// 1. Known, and always there. The compiler checks every call.
        #[child]
        pub fn kiln(&self) -> &Kiln {
            &self.kiln
        }

        /// 2. Known, one per key. Which keys exist is asked while running.
        #[child(list = "shelves")]
        pub fn shelf(&self, number: u32) -> Result<Kiln, NotFound> {
            if number < 4 { Ok(Kiln) } else { Err(NotFound::NoSuch(number.to_string())) }
        }

        /// 3. The interface is known; which service fills it is not. It
        /// may be here or on another machine. Checked once, when it is
        /// connected, by its hash.
        #[child(list = "suppliers")]
        pub fn supplier(&self, name: &str) -> Result<Box<dyn Glazes>, NotFound> {
            Err(NotFound::NoSuch(name.to_owned()))
        }

        /// 4. Nothing is known. Its IR is read while running, and it is
        /// called by name, with JSON.
        #[child(list = "plugins")]
        pub fn plugin(&self, name: &str) -> Result<Box<dyn Service>, NotFound> {
            Err(NotFound::NoSuch(name.to_owned()))
        }

        pub fn shelves(&self) -> Vec<u32> {
            vec![1, 2, 3]
        }
        pub fn suppliers(&self) -> Vec<String> {
            vec![]
        }
        pub fn plugins(&self) -> Vec<String> {
            vec![]
        }
    }
}
