extern crate glitch_in_the_matrix as gm;
extern crate futures;
extern crate tokio_core;
extern crate rpassword;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use gm::MatrixClient;
use gm::room::RoomExt;
use gm::types::messages::{Message};
use gm::types::content::{Content};
use gm::sync::SyncStream;
use gm::request::MatrixRequestable;
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
    let mut mx = core.run(MatrixClient::login_password(username, password, server, &hdl)).unwrap();
    println!("[+] Connected to {} as {}", server, username);
    let ss = SyncStream::new(mx.clone());
    // We discard the results of the initial `/sync`, because we only want to echo
    // new requests.
    let fut = ss.skip(1).for_each(|sync| {
        for (room, evt) in sync.iter_events() {
            if let Some(ref rd) = evt.room_data {
                // only echo messages from other users
                if rd.sender == mx.get_user_id() {
                    continue;
                }
                // tell the server we have read the event
                let mut rc = room.cli(&mut mx);
                hdl.spawn(rc.read_receipt(&rd.event_id).map(|_| ()).map_err(|_| ()));
                if let Content::RoomMessage(ref m) = evt.content {
                    if let Message::Text { ref body, .. } = *m {
                        hdl.spawn(rc.send_simple(body.to_owned()).map(|_| ()).map_err(|_| ()));
                    }
                }
            }
        }
        Ok(())
    });
    core.run(fut).unwrap();
}
