#![cfg(unix)]

use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use futures::StreamExt;
use plexus::tls::{Dial, Identity};
use plexus::{Connection, WINDOW};
use plexus_ir::Ir;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

const STUDIO_KILN: &str = env!("CARGO_BIN_EXE_studio-kiln");
const HEAT: &str = "studio[tenant=acme].kiln.heat";
const AUDIT: &str = "studio[tenant=acme].kiln.audit";

fn key_file(path: &Path) -> Identity {
    use std::os::unix::fs::OpenOptionsExt;

    let mut seed = [0; 32];
    getrandom::fill(&mut seed).unwrap();
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .unwrap()
        .write_all(&seed)
        .unwrap();
    Identity::from_seed(seed).unwrap()
}

async fn launch(mut command: Command) -> (Child, String) {
    let mut child = command
        .env_remove("PLEXUS_DATA")
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut url = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut url)
        .await
        .unwrap();
    assert!(url.starts_with("wss://127.0.0.1:"), "startup output: {url}");
    (child, url.trim().into())
}

async fn connect(url: &str, id: &Identity, peer: &Identity) -> Connection {
    let dial = Dial::new(id, peer.principal()).unwrap();
    let (channel, principal) = dial.connect(url).await.unwrap();
    assert_eq!(principal, peer.principal());
    let (connection, driver) = Connection::new(channel, None);
    tokio::spawn(driver);
    connection.ready().await.unwrap();
    connection
}

async fn read(connection: &Connection, address: &str) -> Value {
    let raw = connection.call(address, ()).await.unwrap();
    serde_json::from_str(raw.get()).unwrap()
}

async fn exercise(
    alice: &Connection,
    bob: &Connection,
    observer: &Connection,
    audit: &str,
    expected_child_caller: &str,
    kiln_hash: &Value,
) -> Vec<Value> {
    assert_eq!(
        read(alice, "studio[tenant=acme].kiln._hash").await,
        *kiln_hash
    );
    let ir = alice
        .call("studio[tenant=acme].kiln._ir", ())
        .await
        .unwrap();
    let ir = Ir::read(ir.get()).unwrap();
    assert_eq!(ir.hash(), kiln_hash.as_str().unwrap());
    let params = || json!({"cone": 6, "hours": 1.0});
    let before = read(observer, audit).await;
    assert_eq!(
        before,
        json!({"started": 0, "readings": 0, "completed": 0, "cancelled": 0})
    );
    let mut denied = bob.stream(HEAT, params()).unwrap();
    assert_eq!(
        denied.next().await.unwrap().unwrap_err().code(),
        Some("not-a-member")
    );
    assert!(denied.next().await.is_none());
    assert_eq!(read(observer, audit).await, before);

    let mut finite = alice.stream(HEAT, params()).unwrap();
    let mut readings = Vec::new();
    while let Some(item) = finite.next().await {
        readings.push(serde_json::from_str::<Value>(item.unwrap().get()).unwrap());
    }
    assert_eq!(
        readings
            .iter()
            .map(|r| r["minute"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        (0..=60).step_by(5).collect::<Vec<_>>()
    );
    assert_eq!(
        readings[0],
        json!({"minute": 0, "celsius": 20.0, "phase": "climbing"})
    );
    assert_eq!(
        readings[8],
        json!({"minute": 40, "celsius": 1168.0, "phase": "holding"})
    );
    assert_eq!(
        readings[12],
        json!({"minute": 60, "celsius": 20.0, "phase": "cooling"})
    );
    assert_eq!(
        read(observer, audit).await,
        json!({"caller": expected_child_caller, "started": 1, "readings": 13, "completed": 1, "cancelled": 0})
    );

    let mut long = alice
        .stream(HEAT, json!({"cone": 6, "hours": 1000.0}))
        .unwrap();
    for minute in [0, 5, 10] {
        let item = long.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(item.get()).unwrap()["minute"],
            minute
        );
    }
    let active = read(observer, audit).await;
    assert_eq!(active["started"], 2);
    assert_eq!(active["cancelled"], 0);
    assert!(active["readings"].as_u64().unwrap() <= 13 + u64::from(WINDOW));
    drop(long);

    let stopped = loop {
        let state = read(observer, audit).await;
        if state["cancelled"] == 1 {
            break state;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    };
    assert_eq!(stopped["started"], 2);
    assert_eq!(stopped["completed"], 1);
    assert_eq!(stopped["caller"], expected_child_caller);
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(read(observer, audit).await, stopped);
    readings
}

#[tokio::test(flavor = "multi_thread")]
async fn the_same_tenant_call_reaches_either_kiln() {
    tokio::time::timeout(Duration::from_secs(15), prove())
        .await
        .expect("Studio/Kiln run did not complete");
}

async fn prove() {
    let directory = tempfile::tempdir().unwrap();
    let studio_file = directory.path().join("studio.key");
    let kiln_file = directory.path().join("kiln.key");
    let studio = key_file(&studio_file);
    let kiln = key_file(&kiln_file);
    let alice = Identity::generate().unwrap();
    let bob = Identity::generate().unwrap();

    let mut kiln_command = Command::new(STUDIO_KILN);
    kiln_command
        .args(["--wss", "127.0.0.1:0", "--key-file"])
        .arg(&kiln_file)
        .args(["--allow-key", studio.principal()]);
    let (kiln_process, kiln_url) = launch(kiln_command).await;
    let kiln_observer = connect(&kiln_url, &studio, &kiln).await;
    let kiln_hash = read(&kiln_observer, "kiln._hash").await;

    let studio_command = || {
        let mut command = Command::new(STUDIO_KILN);
        command
            .arg("studio")
            .arg(&studio_file)
            .args([alice.principal(), bob.principal()]);
        command
    };
    let (mut local_process, local_url) = launch(studio_command()).await;
    let local_alice = connect(&local_url, &alice, &studio).await;
    let local_bob = connect(&local_url, &bob, &studio).await;
    let local_studio_hash = local_alice.ready().await.unwrap();
    let local_tree = read(&local_alice, "_tree").await;
    let local_readings = exercise(
        &local_alice,
        &local_bob,
        &local_alice,
        AUDIT,
        alice.principal(),
        &kiln_hash,
    )
    .await;
    local_process.kill().await.unwrap();

    let wrong_pin = studio_command()
        .args([&kiln_url, alice.principal()])
        .kill_on_drop(true)
        .output()
        .await
        .unwrap();
    assert!(!wrong_pin.status.success());
    assert!(
        wrong_pin.stdout.is_empty(),
        "Studio announced a listener with a bad kiln pin"
    );
    let error = String::from_utf8(wrong_pin.stderr).unwrap();
    assert!(
        error.contains("server offered") && error.contains(alice.principal()),
        "{error}"
    );

    let mut remote_command = studio_command();
    remote_command.args([&kiln_url, kiln.principal()]);
    let (remote_process, remote_url) = launch(remote_command).await;
    assert_ne!(remote_process.id(), kiln_process.id());
    let remote_alice = connect(&remote_url, &alice, &studio).await;
    let remote_bob = connect(&remote_url, &bob, &studio).await;
    assert_eq!(remote_alice.ready().await.unwrap(), local_studio_hash);
    assert_eq!(read(&remote_alice, "_tree").await, local_tree);
    let remote_readings = exercise(
        &remote_alice,
        &remote_bob,
        &kiln_observer,
        "kiln.audit",
        studio.principal(),
        &kiln_hash,
    )
    .await;
    assert_eq!(remote_readings, local_readings);
}
