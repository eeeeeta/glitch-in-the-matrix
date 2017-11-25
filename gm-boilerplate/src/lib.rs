extern crate glitch_in_the_matrix as gm;
extern crate futures;
extern crate tokio_core;

use std::rc::Rc;
use std::cell::RefCell;
use futures::{Future, Stream};
use gm::types::events::Event;
use gm::errors::*;
use gm::types::sync::SyncReply;
use gm::room::{Room, RoomExt};
use gm::{MatrixClient, MatrixFuture};
use tokio_core::reactor::Handle;

pub type Mx = Rc<RefCell<MatrixClient>>;

pub fn sync_boilerplate<Fut, F, C>(mx: Mx, reply: SyncReply, mut f: F) -> MatrixFuture<Vec<(Room<'static>, C)>>
    where Fut: Future<Item=Vec<C>, Error=MatrixError> + 'static,
          F: FnMut(Mx, &Room<'static>, &Event) -> Fut,
          C: 'static
{
    let mut futs: Vec<Box<Future<Item=Vec<(Room<'static>, C)>, Error=MatrixError>>> = vec![];
    for (room, evt) in reply.iter_events() {
        let rc = room.clone();
        if let Event::Full(ref meta, _) = *evt {
            futs.push(Box::new(
                rc.cli(&mut mx.borrow_mut())
                    .read_receipt(&meta.event_id)
                    .map(|_| vec![]))
            );
        }
        futs.push(Box::new(
            f(mx.clone(), room, evt)
                .map(move |x| x.into_iter()
                     .map(|x| (rc.clone(), x))
                     .collect::<Vec<_>>()
                )
        ));
    }
    Box::new(futures::future::join_all(futs.into_iter())
             .map(|vv| {
                 let mut ret = vec![];
                 for vec in vv {
                     ret.extend(vec.into_iter());
                 }
                 ret
             }))
}
pub trait MatrixBot {
    type Command;
    type SyncFuture: Future<Item=Vec<(Room<'static>, Self::Command)>, Error=MatrixError>;
    type CmdFuture: Future<Item=(), Error=MatrixError>;
    type ErrorFuture: Future<Item=(), Error=MatrixError>;

    fn on_login(&mut self, mx: Mx);
    fn on_sync(&mut self, reply: SyncReply) -> Self::SyncFuture;
    fn on_command(&mut self, room: Room<'static>, command: Self::Command) -> Self::CmdFuture;
    fn on_error(&mut self, room: Option<Room<'static>>, error: MatrixError) -> Self::ErrorFuture;
}
pub struct BoilerplateConfig {
    pub server: String,
    pub username: String,
    pub password: String,
}
pub fn make_bot_future<B>(hdl: Handle, cfg: BoilerplateConfig, mut bot: B) -> MatrixFuture<()>
    where B: MatrixBot + 'static {
    let login = MatrixClient::login(&cfg.username, &cfg.password, &cfg.server, &hdl);
    let fut = login.and_then(move |mx| {
        let ss = mx.get_sync_stream();
        let mx = Rc::new(RefCell::new(mx));
        bot.on_login(mx.clone());
        let bot = Rc::new(RefCell::new(bot));
        ss.skip(1).for_each(move |sync| {
            let bot2 = bot.clone();
            let bot4 = bot.clone();
            let fut = {
                let mut bot = bot.borrow_mut();
                bot.on_sync(sync)
            };
            fut
                .or_else(move |err| {
                    let mut bot = bot4.borrow_mut();
                    bot.on_error(None, err)
                        .map(|_| vec![])
                })
                .and_then(move |commands| {
                    let mut futs = vec![];
                    for (room, cmd) in commands {
                        let r2 = room.clone();
                        let fut = {
                            let mut bot = bot2.borrow_mut();
                            bot.on_command(room, cmd)
                        };
                        let bot3 = bot2.clone();
                        let fut = fut.or_else(move |err| {
                            let mut bot = bot3.borrow_mut();
                            bot.on_error(Some(r2), err)
                        });
                        futs.push(Box::new(fut));
                    }
                    futures::future::join_all(futs.into_iter())
                        .map(|_| ())
                        .map_err(|e| e.into())
                })
        })
    });
    Box::new(fut)
}
