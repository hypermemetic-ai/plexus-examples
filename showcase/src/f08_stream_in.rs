/// A stream in: `Input<T>` is what the caller sends while the call runs.
/// It is not one of the parameters.
#[plexus::service(describe)]
pub mod media {
    use plexus::{Bytes, Input};

    pub struct Kept {
        pub sha256: String,
        pub bytes: u64,
    }

    pub struct Media;

    impl Media {
        /// Keep a file. The caller streams its bytes in.
        pub fn put(&self, name: String, file: Input<Bytes>) -> Kept {
            let bytes = file.map(|chunk| chunk.len() as u64).sum();
            Kept { sha256: format!("sha256-of-{name}"), bytes }
        }

        /// Count a run's tokens: a stream in of a type of its own.
        pub fn tokens(&self, run: String, tokens: Input<String>) -> u32 {
            let _ = run;
            tokens.count() as u32
        }
    }
}
