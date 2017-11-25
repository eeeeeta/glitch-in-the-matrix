extern crate glitch_in_the_matrix as matrix_api;
extern crate serde_json;

use matrix_api::types::sync::SyncReply;
use matrix_api::types::events::Event;

use std::fs;
use std::io;
use std::path;

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
    let rd: fs::ReadDir = fs::read_dir("tests/event-examples").unwrap();
    // tunrs a ReadDir into a vec of PathBuf
    let mut paths: Vec<path::PathBuf> = rd.map(|entry| {entry.unwrap().path()}).collect();
    paths.sort();
    println!("test deser_events: trying to parse events");
    for path in paths.iter() {
        let filename = path.file_name().unwrap();
        let filename = filename.to_str().unwrap();
        let path = path.to_str().unwrap();
        let text = read_file(&path);
        let parsed = ::serde_json::from_str::<Event>(&text);
        print!("test deser_events: parsing {:32} ", filename);
        match parsed {
            Ok(ev) => {
                print!("which is ");
                match ev {
                    Event::Minimal(..) => {
                        println!("MinimalEvent");
                    },
                    Event::Redacted(..) => {
                        println!("RedactEvent");
                    },
                    Event::Full(..) => {
                        println!("Event");
                    },
                    o => {
                        failed = true;
                        println!("Errored event: {:?}", o);
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
    for v in ["unstable","r0"].iter() {
        let sync_json = read_file(&format!("tests/sync_{}.json",v));
        println!("test deser_sync: trying to parse sync {}",v);
        ::serde_json::from_str::<SyncReply>(&sync_json).unwrap();
        println!("test deser_sync: sucessfully parsed sync {}!",v);
    }
}
