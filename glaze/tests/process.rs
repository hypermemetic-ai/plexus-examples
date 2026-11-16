//! native › An activation is served by a process: the `glaze` binary over
//! its stdin and stdout.

use glaze::glaze::HASH;
use plexus::Connection;
use serde_json::{Value, json};
use tokio::process::Command;

#[tokio::test]
async fn glaze_as_a_process() {
    let (_child, channel) = plexus::stdio::child(Command::new(env!("CARGO_BIN_EXE_glaze")).kill_on_drop(true)).unwrap();
    let (conn, driving) = Connection::new(channel, Some(HASH));
    tokio::spawn(driving);
    let parse = |raw: plexus::Raw| serde_json::from_str::<Value>(raw.get()).unwrap();
    assert_eq!(conn.ready().await.unwrap(), HASH);
    assert_eq!(parse(conn.call("glaze.recipes", ()).await.unwrap()).as_array().unwrap().len(), 4);
    assert_eq!(parse(conn.call("glaze.coloured", json!({ "colorant": "iron" })).await.unwrap()), json!([{ "name": "celadon", "colorant": "iron", "cone": 10 }]));
    let batch = parse(conn.call("glaze.weigh", json!({ "recipe": "midnight", "grams": 1000 })).await.unwrap());
    assert_eq!(batch, json!({ "recipe": "midnight", "silica-grams": 582, "flux-grams": 388, "colorant-grams": 30 }));
    let e = conn.call("glaze.weigh", json!({ "recipe": "midnight", "grams": 50 })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("too-small"), Some("50")));
    let e = conn.call("glaze.coloured", json!({ "colorant": "gold" })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("invalid-params"), Some(r#""colorant""#)), "{e}");
}
