use std::cell::Cell;

struct ActiveUpload<'a>(&'a Cell<u32>);

impl<'a> ActiveUpload<'a> {
    fn start(active: &'a Cell<u32>) -> Self {
        active.set(active.get() + 1);
        Self(active)
    }
}

impl Drop for ActiveUpload<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

/// A stream in: `Input<T>` is what the caller sends while the call runs.
/// It is not one of the parameters.
#[plexus::activation]
pub mod media {
    use super::ActiveUpload;
    use futures::StreamExt;
    use plexus::{Bytes, Input};
    use sha2::{Digest, Sha256};
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;

    pub struct Kept {
        pub sha256: String,
        pub bytes: u64,
    }

    pub struct UploadStatus {
        pub received: u64,
        pub active: u32,
    }

    pub enum UploadError {
        InvalidInput(String),
        Storage(String),
    }

    #[derive(Default)]
    pub struct Media {
        files: RefCell<HashMap<String, (Kept, File)>>,
        received: Cell<u64>,
        active: Cell<u32>,
    }

    impl Media {
        /// Keep a file. The caller streams its bytes in.
        pub async fn put(&self, name: String, file: Input<Bytes>) -> Result<Kept, UploadError> {
            let _upload = ActiveUpload::start(&self.active);
            let mut file = file;
            let mut stored =
                tempfile::tempfile().map_err(|e| UploadError::Storage(e.to_string()))?;
            let mut digest = Sha256::new();
            let mut bytes = 0;
            while let Some(chunk) = file.next().await {
                let chunk = chunk.map_err(|e| UploadError::InvalidInput(e.message))?;
                stored
                    .write_all(&chunk)
                    .map_err(|e| UploadError::Storage(e.to_string()))?;
                digest.update(&chunk);
                bytes += chunk.len() as u64;
                self.received.set(self.received.get() + chunk.len() as u64);
            }
            let kept = Kept {
                sha256: format!("{:x}", digest.finalize()),
                bytes,
            };
            self.files.borrow_mut().insert(name, (kept.clone(), stored));
            Ok(kept)
        }

        /// Count a run's tokens: a stream in of a type of its own.
        pub async fn tokens(&self, run: String, tokens: Input<String>) -> Result<u32, UploadError> {
            let _ = run;
            let mut tokens = tokens;
            let mut n = 0u32;
            while let Some(token) = tokens.next().await {
                token.map_err(|e| UploadError::InvalidInput(e.message))?;
                n = n
                    .checked_add(1)
                    .ok_or_else(|| UploadError::InvalidInput("too many tokens".into()))?;
            }
            Ok(n)
        }

        /// Active uploads and bytes read so far, including unfinished uploads.
        pub fn status(&self) -> UploadStatus {
            UploadStatus {
                received: self.received.get(),
                active: self.active.get(),
            }
        }

        /// The receipt of the last completed upload to this name.
        pub fn receipt(&self, name: String) -> Option<Kept> {
            self.files.borrow().get(&name).map(|(kept, _)| kept.clone())
        }
    }
}
