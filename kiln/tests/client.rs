//! native › its generated client: `ts/kiln.plexus.ts`, its types taken out
//! as plexus-rt's `scripts/kiln.sh` does, on plexus-rt in Node, calls the
//! kiln process over a WebSocket (`tests/client.mjs`).
//!
//! It needs Node 22.13 or later (its WebSocket and `stripTypeScriptTypes`)
//! and `../../plexus-rt/dist/`, which `npm run build` there writes. Without
//! them the test is reported ignored, with why, never passed.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use libtest_mimic::{Arguments, Trial};

const NAME: &str = "the_generated_client_in_node_over_a_websocket";

fn main() {
    let args = Arguments::from_args();
    let trial = match prerequisites() {
        Ok(rt) => Trial::test(NAME, move || run(&rt).map_err(Into::into)),
        Err(why) => {
            eprintln!("SKIPPED {NAME}: {why}");
            Trial::test(NAME, || Err("not run: its prerequisites are missing".into())).with_ignored_flag(true)
        }
    };
    libtest_mimic::run(&args, vec![trial]).exit();
}

fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Node and plexus-rt's built module, or why one is missing.
fn prerequisites() -> Result<PathBuf, String> {
    let ok = Command::new("node")
        .args(["-e", "process.exit(typeof WebSocket === 'function' && typeof require('node:module').stripTypeScriptTypes === 'function' ? 0 : 1)"])
        .status()
        .map_err(|e| format!("no node on PATH ({e})"))?;
    if !ok.success() {
        return Err("this node has no WebSocket or no stripTypeScriptTypes: the test needs Node 22.13 or later".into());
    }
    let dist = here().join("../../plexus-rt/dist");
    for file in ["plexus-rt.js", "plexus_proto_web.wasm"] {
        if !dist.join(file).exists() {
            return Err(format!("no {} (npm run build in ../../plexus-rt writes it)", dist.join(file).display()));
        }
    }
    Ok(dist.join("plexus-rt.js").canonicalize().map_err(|e| e.to_string())?)
}

struct Killed(Child);

impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn run(rt: &Path) -> Result<(), String> {
    // The client as plexus-rt's scripts/kiln.sh makes it for a page: the
    // types taken out, importing the built runtime.
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("kiln-client-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let client = dir.join("kiln.plexus.js");
    let rt_url = format!("file://{}", rt.display());
    let stripped = Command::new("node")
        .args(["--no-warnings", "-e", r#"
            const { stripTypeScriptTypes } = require("node:module"), fs = require("node:fs");
            const [from, to, rt] = process.argv.slice(1);
            fs.writeFileSync(to, stripTypeScriptTypes(fs.readFileSync(from, "utf8")).replace('"./plexus-rt.ts"', JSON.stringify(rt)));
        "#])
        .arg(here().join("../ts/kiln.plexus.ts"))
        .arg(&client)
        .arg(&rt_url)
        .output()
        .map_err(|e| format!("node did not run: {e}"))?;
    if !stripped.status.success() {
        return Err(format!("could not take the types out of ts/kiln.plexus.ts:\n{}", String::from_utf8_lossy(&stripped.stderr)));
    }
    let mut kiln = Killed(Command::new(env!("CARGO_BIN_EXE_kiln")).args(["--ws", "127.0.0.1:0"]).stdout(Stdio::piped()).spawn().map_err(|e| format!("the kiln did not start: {e}"))?);
    let mut url = String::new();
    BufReader::new(kiln.0.stdout.take().expect("piped")).read_line(&mut url).map_err(|e| e.to_string())?;
    let node = Command::new("node")
        .arg(here().join("tests/client.mjs"))
        .arg(url.trim())
        .arg(format!("file://{}", client.display()))
        .arg(&rt_url)
        .output()
        .map_err(|e| format!("node did not run: {e}"))?;
    let _ = std::fs::remove_dir_all(&dir);
    if node.status.success() {
        return Ok(());
    }
    Err(format!("{}\n{}", String::from_utf8_lossy(&node.stdout), String::from_utf8_lossy(&node.stderr)))
}
