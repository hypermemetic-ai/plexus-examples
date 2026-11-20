//! A tenant gate in front of a kiln held here or in a separate TLS process.

use std::cell::Cell;
use std::io;
use std::sync::Arc;

use plexus::tls::{Dial, Identity, Listen};
use plexus::{Remote, Server};

struct Firing<'a> {
    cancelled: &'a Cell<u32>,
    finished: bool,
}

impl Firing<'_> {
    fn finish(mut self) {
        self.finished = true;
    }
}

impl Drop for Firing<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.cancelled.set(self.cancelled.get() + 1);
        }
    }
}

/// Instrumentation for this example. The kiln's heating calculation is shared
/// with the ordinary kiln binary.
#[plexus::activation]
mod kiln {
    use super::Firing;
    use ::kiln::kiln::{Kiln as Heater, Phase as HeaterPhase};
    use futures::{Stream, StreamExt};
    use plexus::Caller;
    use std::cell::{Cell, RefCell};

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

    pub struct Observation {
        pub caller: Option<String>,
        pub started: u32,
        pub readings: u32,
        pub completed: u32,
        pub cancelled: u32,
    }

    #[derive(Default)]
    pub struct Kiln {
        heater: Heater,
        caller: RefCell<Option<String>>,
        started: Cell<u32>,
        readings: Cell<u32>,
        completed: Cell<u32>,
        cancelled: Cell<u32>,
    }

    impl Kiln {
        /// Temperature readings from the example kiln.
        pub fn heat(&self, cone: u8, hours: f32, caller: &Caller) -> impl Stream<Item = Reading> {
            self.started.set(self.started.get() + 1);
            self.caller.replace(Some(caller.principal().into()));
            let firing = Firing {
                cancelled: &self.cancelled,
                finished: false,
            };
            futures::stream::unfold(
                (Box::pin(self.heater.heat(cone, hours)), firing),
                move |(mut heat, firing)| async move {
                    if let Some(reading) = heat.next().await {
                        let count = &self.readings;
                        count.set(count.get() + 1);
                        let phase = match reading.phase {
                            HeaterPhase::Climbing => Phase::Climbing,
                            HeaterPhase::Holding => Phase::Holding,
                            HeaterPhase::Cooling => Phase::Cooling,
                        };
                        let reading = Reading {
                            minute: reading.minute,
                            celsius: reading.celsius,
                            phase,
                        };
                        Some((reading, (heat, firing)))
                    } else {
                        firing.finish();
                        let count = &self.completed;
                        count.set(count.get() + 1);
                        None
                    }
                },
            )
        }

        /// Process-local observations of heat calls for the example.
        pub fn audit(&self) -> Observation {
            Observation {
                caller: self.caller.borrow().clone(),
                started: self.started.get(),
                readings: self.readings.get(),
                completed: self.completed.get(),
                cancelled: self.cancelled.get(),
            }
        }
    }
}

#[plexus::activation(context = StudioCx)]
mod studio {
    use plexus::Caller;

    #[derive(plexus::Context)]
    pub struct StudioCx {
        #[from(caller)]
        caller: Caller,
        tenant: String,
    }

    pub enum Denied {
        NotAMember,
    }

    pub struct Studio {
        pub alice: String,
    }

    impl Studio {
        #[open]
        pub fn open(&self, cx: &StudioCx) -> Result<(), Denied> {
            if cx.tenant == "acme" && cx.caller.principal() == self.alice {
                Ok(())
            } else {
                Err(Denied::NotAMember)
            }
        }

        pub fn name(&self) -> String {
            "studio".into()
        }
    }
}

fn main() -> io::Result<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|arg| arg == "--wss") {
        return plexus::main(kiln::Kiln::default());
    }
    if let [mode, key] = arguments.as_slice()
        && mode == "principal"
    {
        println!("{}", Identity::from_key_file(key)?.principal());
        return Ok(());
    }
    let [mode, key, alice, bob, remote @ ..] = arguments.as_slice() else {
        return Err(usage());
    };
    if mode != "studio" {
        return Err(usage());
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let identity = Identity::from_key_file(key)?;
        let listen = Listen::new(&identity, [alice.as_str(), bob.as_str()])?;
        let mut server = Server::new();
        match remote {
            [] => kiln::serve(kiln::Kiln::default(), &mut server, "studio.kiln"),
            [url, pin] => {
                let kiln = Remote::wss(url, "kiln", Dial::new(&identity, pin)?)?;
                server.mount("studio.kiln", Box::new(kiln));
            }
            _ => return Err(usage()),
        }
        studio::serve(
            studio::Studio {
                alice: alice.clone(),
            },
            &mut server,
            "studio",
        );
        let bound = listen
            .spawn("127.0.0.1:0".parse().unwrap(), Arc::new(server))
            .await?;
        println!("wss://{bound}/");
        std::future::pending().await
    })
}

fn usage() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "usage: studio-kiln principal KEY_FILE\n       studio-kiln --wss ADDR --key-file KEY_FILE --allow-key STUDIO_PIN\n       studio-kiln studio KEY_FILE ALICE_PIN BOB_PIN [KILN_WSS_URL KILN_PIN]",
    )
}
