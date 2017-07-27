extern crate glitch_in_the_matrix as gm;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use gm::{MatrixClient, MatrixFuture};
use gm::types::{Content, Message};
use std::env;

fn main() {
    let mut core = Core::new().unwrap();
    let hdl = core.handle();
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: cargo run --example echo -- SERVER USERNAME PASSWORD");
        return;
    }
    let (server, username, password) = (&args[0], &args[1], &args[2]);
    println!("server: {}\nusername: {}\npassword: {}", server, username, password);
    let mut mx = core.run(MatrixClient::login(username, password, server, &hdl)).unwrap();
    println!("[+] Connected to Matrix.");
    let mut ss = mx.get_sync_stream();
    let mut first = true;
    let fut = ss.for_each(|sync| {
        let mut futs: Vec<MatrixFuture<()>> = vec![];
        if !first {
            // We discard the results of the initial `/sync`, because we only want to echo
            // new requests.
            for (rid, room) in sync.rooms.join {
                for event in room.timeline.events {
                    match event.content {
                        Content::RoomMessage(m) => {
                            if let Message::Text { body, .. } = m {
                                if event.sender == mx.user_id() {
                                    continue;
                                }
                                futs.push(Box::new(mx.send_simple(&rid, body).map(|_| ())));
                            }
                        },
                        _ => {}
                    }
                    futs.push(Box::new(mx.read_receipt(&rid, &event.event_id).map(|_| ())));
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
