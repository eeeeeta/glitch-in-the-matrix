extern crate glitch_in_the_matrix as gm;
extern crate futures;
extern crate tokio_core;
extern crate rpassword;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use gm::{MatrixClient, MatrixFuture};
use gm::types::messages::{Message};
use gm::types::content::{Content};
use gm::types::events::{EventMetadata, Event};
use rpassword::prompt_password_stdout;
use std::env;


fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: cargo run --example echo -- SERVER USERNAME");
        return;
    }
    let (server, username) = (&args[0], &args[1]);
    println!("Type password for the bot (characters won't show up as you type them)");
    let password = &prompt_password_stdout("password:").unwrap();
    let mut core = Core::new().unwrap();
    let hdl = core.handle();
    let mut mx = core.run(MatrixClient::login(username, password, server, &hdl)).unwrap();
    println!("[+] Connected to {} as {}", server, username);
    let ss = mx.get_sync_stream();
    // We discard the results of the initial `/sync`, because we only want to echo
    // new requests.
    let fut = ss.skip(1).for_each(|sync| {
        let mut futs: Vec<MatrixFuture<()>> = vec![];
        for (room, events) in sync.rooms.join {
            for Event(meta, content) in events.timeline.events {
                if let EventMetadata::Full(meta) = meta {
                    // only echo messages from other users
                    if meta.sender == mx.user_id() {
                        continue;
                    }
                    // tell the server we have read the event
                    let mut rc = room.cli(&mut mx);
                    futs.push(Box::new(rc.read_receipt(&meta.event_id).map(|_| ())));
                    if let Content::RoomMessage(m) = content {
                        if let Message::Text { body, .. } = m {
                            futs.push(Box::new(rc.send_simple(body).map(|_| ())));
                        }
                    }
                }
            }
        }
        futures::future::join_all(futs.into_iter()).map(|_| ())
    });
    core.run(fut).unwrap();
}
