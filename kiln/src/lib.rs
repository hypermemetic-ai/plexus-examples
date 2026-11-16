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
    use plexus::Store;
    use plexus::rusqlite::{OptionalExtension, Row, params};

    pub struct Kiln {
        store: Store,
    }

    impl Default for Kiln {
        fn default() -> Kiln {
            Kiln { store: Store::open("kiln", "create table if not exists firing (id integer primary key, cone integer not null, hours real not null, loaded_by text not null)") }
        }
    }

    const FIRING: &str = "select id, cone, hours, loaded_by from firing";

    fn firing(row: &Row) -> plexus::rusqlite::Result<Firing> {
        Ok(Firing { id: row.get(0)?, cone: row.get(1)?, hours: row.get::<_, f64>(2)? as f32, loaded_by: row.get(3)? })
    }

    impl Kiln {
        fn one(&self, sql: &str, id: Option<u32>) -> Option<Firing> {
            self.store.query_row(sql, plexus::rusqlite::params_from_iter(id), firing).optional().expect("the kiln's store reads")
        }

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
            let id = self.store
                .query_row("insert into firing (cone, hours, loaded_by) values (?1, ?2, ?3) returning id", params![cone, hours as f64, loaded_by], |r| r.get(0))
                .expect("the kiln's store writes");
            Ok(Firing { id, cone, hours, loaded_by })
        }

        /// Every firing, oldest first.
        pub fn log(&self) -> Vec<Firing> {
            let mut all = self.store.prepare_cached(&format!("{FIRING} order by id")).expect("the kiln's store reads");
            all.query_map([], firing).and_then(Iterator::collect).expect("the kiln's store reads")
        }

        /// One firing by its id.
        pub fn get(&self, id: u32) -> Result<Firing, KilnError> {
            self.one(&format!("{FIRING} where id = ?1"), Some(id)).ok_or(KilnError::NotFound(id))
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
            self.one(&format!("{FIRING} order by cone desc, id desc limit 1"), None)
        }
    }
}
