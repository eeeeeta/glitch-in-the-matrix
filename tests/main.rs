extern crate glitch_in_the_matrix as matrix_api;
extern crate serde_json;

use matrix_api::types::{SyncReply,EventTypes};
use std::fs;
use std::io;

fn read_file(file: &str) -> String {
    let mut text = String::new();
    let mut f = fs::File::open(&file)
        .expect(&format!("File not found: '{}'!",file));
    io::Read::read_to_string(&mut f,&mut text).expect("something went wrong reading the file");
    text

}

#[test]
fn deser_events() {
    let mut failed = false;
    let paths = fs::read_dir("tests/event-examples").unwrap();
    println!("test deser_events: trying to parse events");
    for path in paths {
        let path = path.unwrap();
        let filename = path.file_name();
        let filename = filename.to_str().unwrap();
        let path = path.path();
        let path = path.to_str().unwrap();
        let text = read_file(&path);
        let parsed = ::serde_json::from_str::<EventTypes>(&text);
        print!("test deser_events: parsing {:32} which is ", filename);
        match parsed {
            Ok(res) => {
                match res {
                    EventTypes::EphemeralEvent(_) => {
                        println!("EphemeralEvent");
                    },
                    EventTypes::InviteStateEvent(_) => {
                        println!("InviteStateEvent");
                    }
                    EventTypes::RedactedEvent(_) => {
                        println!("RedactEvent");
                    }
                    EventTypes::Event(_) => {
                        println!("Event");
                    }
                }
            },
            Err(error) => {
                failed = true;
                println!("Error {}",error);
            }
        }
    }
    if failed {
        panic!("failed to parse one or more examples");
    } else {
        println!("test deser_events: sucessfully parsed all events");
    }
}


#[test]
fn deser_sync() {
    for v in ["unstable","r0"] {
        let sync_json = read_file(format!("tests/{}_sync.json",v));
        println!("test deser_sync: trying to parse sync {}",v);
        ::serde_json::from_str::<SyncReply>(&sync_json).unwrap();
        println!("test deser_sync: sucessfully parsed sync {}!",v);
    }
}
