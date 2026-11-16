//! held › A child is reached under its parent: `#[activation]` serving each
//! way of holding, in memory. A test activation, `meter`, counts what its
//! stream made and dropped, which pacing and the cancel through a parent
//! held in this process are read from (the kiln keeps no such count). It is
//! held two deep, `yard.bench.meter`, beside a child behind an interface:
//! the glaze activation, filling `Box<dyn Glazes>`.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use plexus::{Connection, Remote, Server, TOP_UP, WINDOW, pair};
use serde_json::{Value, json};

/// Counts a stream as dropped when it is.
struct Watched<'a>(&'a std::cell::Cell<u32>);

impl Drop for Watched<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

/// Counts its readings.
#[plexus::activation]
pub mod meter {
    use super::Watched;
    use futures::Stream;
    use std::cell::Cell;

    #[derive(Default)]
    pub struct Meter {
        made: Cell<u32>,
        dropped: Cell<u32>,
    }

    impl Meter {
        /// Readings, for as long as they are read.
        pub fn tick(&self) -> impl Stream<Item = u32> {
            futures::stream::unfold((0, Watched(&self.dropped)), move |(n, w)| async move {
                self.made.set(self.made.get() + 1);
                Some((n, (n + 1, w)))
            })
        }

        /// How many readings were made, and how many streams dropped.
        pub fn count(&self) -> (u32, u32) {
            (self.made.get(), self.dropped.get())
        }
    }
}

/// A bench: the meter in this process, a glaze supplier behind an interface.
#[plexus::activation]
pub mod bench {
    use super::meter::Meter;
    use plexus::Remote;

    /// Glaze recipes: whatever serves them.
    pub trait Glazes: plexus::Activation {}
    impl Glazes for Remote {}

    pub struct Bench {
        pub meter: Meter,
        pub supplier: Remote,
    }

    impl Bench {
        pub fn name(&self) -> String {
            "bench".into()
        }

        #[child]
        pub fn meter(&self) -> &Meter {
            &self.meter
        }

        #[child]
        pub fn supplier(&self) -> Box<dyn Glazes> {
            Box::new(self.supplier.clone())
        }
    }
}

/// A yard, holding a bench.
#[plexus::activation]
pub mod yard {
    use super::bench::Bench;

    pub struct Yard {
        pub bench: Bench,
    }

    impl Yard {
        #[child]
        pub fn bench(&self) -> &Bench {
            &self.bench
        }
    }
}

fn parse(raw: &plexus::Raw) -> Value {
    serde_json::from_str(raw.get()).unwrap()
}

fn connect(server: Server) -> Connection {
    let (near, far) = pair();
    tokio::spawn(Arc::new(server).serve(far));
    let (conn, driving) = Connection::new(near, None);
    tokio::spawn(driving);
    conn
}

/// The glaze activation, served in memory, held across that channel.
fn glaze() -> Remote {
    let mut server = Server::new();
    glaze::glaze::serve(glaze::glaze::Glaze, &mut server, "glaze");
    Remote::new(connect(server), "glaze")
}

async fn count(conn: &Connection) -> (u64, u64) {
    let v = parse(&conn.call("yard.bench.meter.count", ()).await.unwrap());
    (v[0].as_u64().unwrap(), v[1].as_u64().unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn each_way_of_holding() {
    let mut server = Server::new();
    let yard = yard::Yard { bench: bench::Bench { meter: meter::Meter::default(), supplier: glaze() } };
    yard::serve(yard, &mut server, "yard");
    let conn = connect(server);
    assert_eq!(conn.ready().await.unwrap(), yard::HASH, "hello says the root's interface");

    // In this process, two deep; behind an interface, across a channel.
    assert_eq!(parse(&conn.call("yard.bench.name", ()).await.unwrap()), json!("bench"));
    assert_eq!(parse(&conn.call("yard.bench.meter.count", ()).await.unwrap()), json!([0, 0]));
    assert_eq!(parse(&conn.call("yard.bench.supplier.recipes", ()).await.unwrap()).as_array().unwrap().len(), 4);
    let e = conn.call("yard.bench.supplier.weigh", json!({ "recipe": "glass", "grams": 5 })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("too-small"), Some("5")));

    // Every place, with its hash; each `_ir` answers.
    let tree = parse(&conn.call("_tree", ()).await.unwrap());
    assert_eq!(
        tree,
        json!([
            { "place": "yard", "hash": yard::HASH },
            { "place": "yard.bench", "hash": bench::HASH },
            { "place": "yard.bench.meter", "hash": meter::HASH },
            { "place": "yard.bench.supplier", "hash": glaze::glaze::HASH },
        ])
    );
    assert_eq!(conn.call("yard.bench.meter._ir", ()).await.unwrap().get(), meter::IR.trim_end());
    assert_eq!(conn.call("yard.bench.supplier._ir", ()).await.unwrap().get(), glaze::glaze::IR.trim_end());
    assert!(bench::IR.contains(r#""interface": "glazes""#), "{}", bench::IR);

    // A stream through the parents, read slowly: paced end to end.
    let mut items = conn.stream("yard.bench.meter.tick", ()).unwrap();
    let mut furthest = 0;
    for taken in 1..=TOP_UP as u64 + 8 {
        assert_eq!(parse(&items.next().await.unwrap().unwrap()), json!(taken - 1));
        tokio::time::sleep(Duration::from_millis(5)).await;
        let ahead = count(&conn).await.0 - taken;
        assert!(ahead <= WINDOW as u64, "the meter made {ahead} more than the caller took");
        furthest = furthest.max(ahead);
    }
    assert_eq!(furthest, WINDOW as u64, "the meter never filled the window");
    // Left early: the cancel reaches the meter, which stops.
    drop(items);
    for _ in 0..2000 {
        if count(&conn).await.1 == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    let (made, dropped) = count(&conn).await;
    assert_eq!(dropped, 1, "the meter's stream was not dropped");
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(count(&conn).await.0, made, "the meter made more after it was dropped");
}
