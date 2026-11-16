//! native › An activation is served by a process: the `kiln` binary, one
//! line of `main`, started as a child and called with `plexus::Connection`
//! over its stdin and stdout, and over a WebSocket.

use std::process::Stdio;
use std::time::Duration;

use futures::StreamExt;
use kiln::kiln::{HASH, IR};
use plexus::Connection;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

fn parse(raw: &plexus::Raw) -> Value {
    serde_json::from_str(raw.get()).unwrap()
}

fn connect(channel: plexus::Channel) -> Connection {
    let (conn, driving) = Connection::new(channel, Some(HASH));
    tokio::spawn(driving);
    conn
}

/// `kiln --ws 127.0.0.1:0`, and the URL it says it bound.
async fn on_a_websocket() -> (Child, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kiln")).args(["--ws", "127.0.0.1:0"]).stdout(Stdio::piped()).kill_on_drop(true).spawn().unwrap();
    let mut url = String::new();
    BufReader::new(child.stdout.take().unwrap()).read_line(&mut url).await.unwrap();
    (child, url.trim().to_owned())
}

/// Every scenario of the ticket, on one connection to a fresh kiln.
async fn scenarios(conn: &Connection) {
    assert_eq!(conn.ready().await.unwrap(), HASH);

    // `kiln._ir` is its IR.
    let ir: Value = serde_json::from_str(IR).unwrap();
    assert_eq!(parse(&conn.call("kiln._ir", ()).await.unwrap()), ir);
    assert_eq!(parse(&conn.call("kiln._hash", ()).await.unwrap()), json!(HASH));

    // fire, log and heat answer, by the value layout.
    let fired = conn.call("kiln.fire", json!({ "cone": 6, "hours": 0.5, "loaded-by": "ada" })).await.unwrap();
    assert_eq!(parse(&fired), json!({ "id": 1, "cone": 6, "hours": 0.5, "loaded-by": "ada" }));
    assert_eq!(parse(&conn.call("kiln.log", ()).await.unwrap()), json!([{ "id": 1, "cone": 6, "hours": 0.5, "loaded-by": "ada" }]));
    assert_eq!(parse(&conn.call("kiln.hottest", ()).await.unwrap())["id"], 1);
    let readings: Vec<Value> = conn.stream("kiln.heat", json!({ "cone": 6, "hours": 1.0 })).unwrap().map(|r| parse(&r.unwrap())).collect().await;
    assert_eq!(readings.len(), 13);
    assert_eq!(readings[0], json!({ "minute": 0, "celsius": 20.0, "phase": "climbing" }));
    assert_eq!(readings[12]["phase"], "cooling");
    // `call` on a stream reads it whole and answers the last.
    assert_eq!(parse(&conn.call("kiln.heat", json!({ "cone": 6, "hours": 1.0 })).await.unwrap()), readings[12]);

    // A typed failure: the case's name, its value, its doc.
    let e = conn.call("kiln.fire", json!({ "cone": 11, "hours": 1.0, "loaded-by": "ada" })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("too-hot"), Some("11")), "{e}");
    assert_eq!(e.to_string(), "too-hot: The kiln goes to cone 10; this asked for more.");
    let e = conn.call("kiln.fire", json!({ "cone": 6, "hours": 1.0, "loaded-by": " " })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("unattended"), None), "{e}");
    let e = conn.call("kiln.get", json!({ "id": 9 })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("not-found"), Some("9")), "{e}");

    // A body that is wrong names the parameter, and the method does not run.
    let e = conn.call("kiln.fire", json!({ "cone": "hot", "hours": 1.0, "loaded-by": "ada" })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("invalid-params"), Some(r#""cone""#)), "{e}");
    assert!(e.to_string().contains("`cone`: invalid type: string \"hot\", expected u8"), "{e}");
    let e = conn.call("kiln.fire", json!({ "cone": 6, "hours": 1.0 })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("invalid-params"), Some(r#""loaded-by""#)), "{e}");
    let e = conn.call("kiln.fire", json!({ "cone": 6, "hours": 1.0, "loaded-by": "ada", "glaze": "celadon" })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("invalid-params"), Some(r#""glaze""#)), "{e}");
    assert_eq!(parse(&conn.call("kiln.log", ()).await.unwrap()).as_array().unwrap().len(), 1, "a refused call fired the kiln");

    // An `async fn` waits while other calls run beside it.
    let long = conn.call("kiln.fire", json!({ "cone": 10, "hours": 20000.0, "loaded-by": "bo" }));
    let log = conn.call("kiln.log", ());
    let (long, log) = futures::join!(long, log);
    assert_eq!(parse(&log.unwrap()).as_array().unwrap().len(), 1, "the log answered after the long firing");
    assert_eq!(parse(&long.unwrap())["id"], 2);
}

#[tokio::test]
async fn over_stdio() {
    let (mut child, channel) = plexus::stdio::child(Command::new(env!("CARGO_BIN_EXE_kiln")).kill_on_drop(true)).unwrap();
    let conn = connect(channel);
    scenarios(&conn).await;
    // Closing its stdin ends the process.
    drop(conn);
    let status = tokio::time::timeout(Duration::from_secs(5), child.wait()).await.expect("the kiln exits").unwrap();
    assert!(status.success(), "{status}");
}

#[tokio::test]
async fn over_a_websocket_every_connection_shares_the_one_kiln() {
    let (_child, url) = on_a_websocket().await;
    let first = connect(plexus::ws::connect(&url).await.unwrap());
    scenarios(&first).await;
    let second = connect(plexus::ws::connect(&url).await.unwrap());
    let log = parse(&second.call("kiln.log", ()).await.unwrap());
    assert_eq!(log.as_array().unwrap().len(), 2, "another connection sees another kiln: {log}");
}

#[tokio::test]
async fn a_reader_that_leaves_cancels_the_stream() {
    let (_child, url) = on_a_websocket().await;
    let conn = connect(plexus::ws::connect(&url).await.unwrap());
    let mut items = conn.stream("kiln.heat", json!({ "cone": 6, "hours": 100000.0 })).unwrap();
    for _ in 0..3 {
        items.next().await.unwrap().unwrap();
    }
    drop(items);
    // The kiln is still there, answering.
    assert_eq!(parse(&conn.call("kiln.log", ()).await.unwrap()), json!([]));
}
