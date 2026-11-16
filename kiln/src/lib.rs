/// A kiln, and the log of what was fired in it.
#[plexus::activation]
pub mod kiln {
    /// One firing.
    pub struct Firing {
        pub id: u32,
        /// How hot: the pyrometric cone, 1 to 10.
        pub cone: u8,
        pub hours: f32,
        pub loaded_by: String,
    }

    pub enum KilnError {
        /// The kiln goes to cone 10; this asked for more.
        TooHot(u8),
        /// No firing has this id.
        NotFound(u32),
        /// Nobody loaded it.
        Unattended,
    }

    /// The kiln's temperature at one moment of a firing.
    pub struct Reading {
        pub minute: u32,
        pub celsius: f32,
        pub phase: Phase,
    }

    pub enum Phase {
        Climbing,
        Holding,
        Cooling,
    }

    use futures::{Stream, StreamExt};
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct Kiln {
        firings: RefCell<Vec<Firing>>,
    }

    impl Kiln {
        /// Fire the kiln and write it in the log. It waits out the whole
        /// firing, reading by reading; other calls run meanwhile.
        pub async fn fire(&self, cone: u8, hours: f32, loaded_by: String) -> Result<Firing, KilnError> {
            if cone > 10 {
                return Err(KilnError::TooHot(cone));
            }
            if loaded_by.trim().is_empty() {
                return Err(KilnError::Unattended);
            }
            let mut readings = std::pin::pin!(self.heat(cone, hours));
            while readings.next().await.is_some() {
                plexus::yield_now().await;
            }
            let mut firings = self.firings.borrow_mut();
            let firing = Firing { id: firings.len() as u32 + 1, cone, hours, loaded_by };
            firings.push(firing.clone());
            Ok(firing)
        }

        /// Every firing, oldest first.
        pub fn log(&self) -> Vec<Firing> {
            self.firings.borrow().clone()
        }

        /// One firing by its id.
        pub fn get(&self, id: u32) -> Result<Firing, KilnError> {
            self.firings.borrow().iter().find(|f| f.id == id).cloned().ok_or(KilnError::NotFound(id))
        }

        /// A firing as it happens: a reading every five minutes, climbing
        /// to the cone's temperature, holding there, then cooling. The
        /// next reading is not worked out until the last one is taken.
        pub fn heat(&self, cone: u8, hours: f32) -> impl Stream<Item = Reading> {
            let peak = 1000.0 + 28.0 * cone.min(10) as f32;
            let total = (hours * 12.0) as u32;
            let (climb, hold) = (total * 6 / 10, total * 7 / 10);
            futures::stream::iter(0..=total).map(move |i| {
                let (celsius, phase) = if i <= climb {
                    (20.0 + (peak - 20.0) * (i as f32 / climb.max(1) as f32).powf(0.7), Phase::Climbing)
                } else if i <= hold {
                    (peak, Phase::Holding)
                } else {
                    let t = (i - hold) as f32 / (total - hold).max(1) as f32;
                    (20.0 + (peak - 20.0) * (1.0 - t).powi(2), Phase::Cooling)
                };
                Reading { minute: i * 5, celsius, phase }
            })
        }

        /// The hottest firing so far, if there was one.
        pub fn hottest(&self) -> Option<Firing> {
            self.firings.borrow().iter().max_by_key(|f| f.cone).cloned()
        }
    }
}
