//! Expose the same media activation directly or through a held child process.
use plexus::{Remote, Server};
use showcase::f08_stream_in::media;
use std::{env, io, sync::Arc};
use tokio::process::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut server = Server::new();
    match env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [mode] if mode == "child" => {
            media::serve(media::Media::default(), &mut server, "media");
            return Arc::new(server).serve(plexus::stdio::stdio()).await;
        }
        [mode] if mode == "local" => {
            media::serve(media::Media::default(), &mut server, "studio.media")
        }
        [mode] if mode == "remote" => {
            // This mount serves the same media interface as the local mode.
            server.hello(media::HASH);
            let mut child = Command::new(env::current_exe()?);
            child.arg("child");
            server.mount("studio.media", Box::new(Remote::child(child, "media")?));
        }
        _ => return Err(io::Error::other("usage: media-input local|remote|child")),
    }
    let addr = plexus::ws::spawn(([127, 0, 0, 1], 0).into(), Arc::new(server)).await?;
    println!("ws://{addr}/");
    std::future::pending().await
}
