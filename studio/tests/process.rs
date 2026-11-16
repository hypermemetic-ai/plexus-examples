//! held › The studio as a process: `studio --ws`, which starts the `glaze`
//! binary beside it as its child, called with `plexus::Connection`. The
//! glaze binary is built first (`cargo build -p glaze --bin glaze`): a
//! dependency's binary is not built for a test.

use std::process::Stdio;
use std::time::Duration;

use futures::StreamExt;
use plexus::Connection;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

fn parse(raw: &plexus::Raw) -> Value {
    serde_json::from_str(raw.get()).unwrap()
}

/// The glaze binary, beside the studio's, where the studio looks by default.
fn build_glaze() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml");
    let built = std::process::Command::new(cargo).args(["build", "-q", "--manifest-path", manifest, "-p", "glaze", "--bin", "glaze"]).status().unwrap();
    assert!(built.success(), "cargo build -p glaze --bin glaze");
}

/// `studio --ws 127.0.0.1:0`, and a connection to the URL it says.
async fn studio() -> (Child, Connection) {
    build_glaze();
    let mut child = Command::new(env!("CARGO_BIN_EXE_studio")).args(["--ws", "127.0.0.1:0"]).stdout(Stdio::piped()).kill_on_drop(true).spawn().unwrap();
    let mut url = String::new();
    BufReader::new(child.stdout.take().unwrap()).read_line(&mut url).await.unwrap();
    let (conn, driving) = Connection::new(plexus::ws::connect(url.trim()).await.unwrap(), Some(studio::studio::HASH));
    tokio::spawn(driving);
    assert_eq!(conn.ready().await.unwrap(), studio::studio::HASH);
    (child, conn)
}

/// The processes `pid` started.
fn children_of(pid: u32) -> Vec<u32> {
    let tasks = std::fs::read_dir(format!("/proc/{pid}/task")).unwrap();
    tasks
        .flat_map(|t| std::fs::read_to_string(t.unwrap().path().join("children")).unwrap_or_default().split_whitespace().map(|p| p.parse().unwrap()).collect::<Vec<u32>>())
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_studio_holds_the_kiln_and_the_glaze() {
    let (child, conn) = studio().await;

    // in process: the kiln the studio holds fires, and the studio sees it.
    let fired = conn.call("studio.kiln.fire", json!({ "cone": 6, "hours": 0.5, "loaded-by": "ada" })).await.unwrap();
    assert_eq!(parse(&fired), json!({ "id": 1, "cone": 6, "hours": 0.5, "loaded-by": "ada" }));
    assert_eq!(parse(&conn.call("studio.fired", ()).await.unwrap()), json!(1));
    let e = conn.call("studio.kiln.get", json!({ "id": 9 })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("not-found"), Some("9")));

    // across a channel: the glaze process answers through the studio.
    let recipes = parse(&conn.call("studio.glaze.recipes", ()).await.unwrap());
    assert_eq!(recipes[0], json!({ "name": "celadon", "colorant": "iron", "cone": 10 }));
    assert_eq!(recipes.as_array().unwrap().len(), 4);
    let e = conn.call("studio.glaze.weigh", json!({ "recipe": "midnight", "grams": 50 })).await.unwrap_err();
    assert_eq!((e.code(), e.data()), (Some("too-small"), Some("50")));

    // learned: every place with its hash, and each `_ir`.
    let tree = parse(&conn.call("_tree", ()).await.unwrap());
    assert_eq!(
        tree,
        json!([
            { "place": "studio", "hash": studio::studio::HASH },
            { "place": "studio.kiln", "hash": kiln::kiln::HASH },
            { "place": "studio.glaze", "hash": glaze::glaze::HASH },
        ])
    );
    // (Over stdio a newline in a frame is a space: the same JSON, other text.)
    let ir = |text: &str| serde_json::from_str::<Value>(text).unwrap();
    assert_eq!(parse(&conn.call("studio.kiln._ir", ()).await.unwrap()), ir(kiln::kiln::IR));
    assert_eq!(parse(&conn.call("studio.glaze._ir", ()).await.unwrap()), ir(glaze::glaze::IR));

    // a stream through the parent, read slowly and left early: the studio
    // goes on answering (pacing and the cancel through a parent are counted
    // in `held.rs` and in plexus's `remote.rs`; the kiln counts neither).
    let mut items = conn.stream("studio.kiln.heat", json!({ "cone": 6, "hours": 100000.0 })).unwrap();
    for minute in [0, 5, 10] {
        assert_eq!(parse(&items.next().await.unwrap().unwrap())["minute"], minute);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    drop(items);
    assert_eq!(parse(&conn.call("studio.kiln.log", ()).await.unwrap()).as_array().unwrap().len(), 1);

    // the child is gone: the glaze process killed.
    let glaze = children_of(child.id().unwrap());
    assert_eq!(glaze.len(), 1, "the studio started one child: {glaze:?}");
    assert!(std::process::Command::new("kill").args(["-9", &glaze[0].to_string()]).status().unwrap().success());
    let mut gone = None;
    for _ in 0..500 {
        match conn.call("studio.glaze.recipes", ()).await {
            Ok(_) => tokio::time::sleep(Duration::from_millis(10)).await,
            Err(e) => {
                gone = Some(e);
                break;
            }
        }
    }
    assert_eq!(gone.expect("glaze still answers").code(), Some("unavailable"));
    // ...and the rest of the tree is untouched.
    assert_eq!(parse(&conn.call("studio.kiln.log", ()).await.unwrap()).as_array().unwrap().len(), 1);
    assert_eq!(parse(&conn.call("studio.fired", ()).await.unwrap()), json!(1));
    let tree = parse(&conn.call("_tree", ()).await.unwrap());
    assert_eq!(tree[2], json!({ "place": "studio.glaze" }), "{tree}");
}
