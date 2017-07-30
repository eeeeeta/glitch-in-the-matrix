extern crate glitch_in_the_matrix as gm;
extern crate futures;
extern crate tokio_core;
extern crate rpassword;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use gm::{MatrixClient, MatrixFuture};
// use gm::types::{EventTypes, Content, Message};
use gm::types::messages::{Message};
use gm::types::content::{Content};
use gm::types::events::{EventTypes};
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
    println!("[+] Connected to {} as {}",server,username);
    let ss = mx.get_sync_stream();
    let mut first = true;
    let fut = ss.for_each(|sync| {
        let mut futs: Vec<MatrixFuture<()>> = vec![];
        if !first {
            // We discard the results of the initial `/sync`, because we only want to echo
            // new requests.
            for (rid, room) in sync.rooms.join {
                for event in room.timeline.events {
                    // we only want messages, so we ignore the other event types
                    if let EventTypes::Event(event) = event {
                        // only echo messages from other users
                        if event.sender == mx.user_id() {
                            continue;
                        }
                        // tell the server we have read the event
                        futs.push(Box::new(mx.read_receipt(&rid, &event.event_id).map(|_| ())));
                        if let Content::Message(m) = event.content {
                            if let Message::Text { body, .. } = m {
                                futs.push(Box::new(mx.send_simple(&rid, body).map(|_| ())));
                            }
                        }
                    }
                }
            }
        }
        else {
            first = false;
        }
        futures::future::join_all(futs.into_iter()).map(|_| ())
    });
    core.run(fut).unwrap();
}
