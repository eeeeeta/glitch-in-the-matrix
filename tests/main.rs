extern crate glitch_in_the_matrix as matrix_api;
extern crate serde_json;

use matrix_api::types::{SyncReply,EventTypes,RedactsEvent};
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
        let parts = filename.split(".").collect::<Vec<&str>>().len();
        if parts != 2 {
            let path = path.path();
            let path = path.to_str().unwrap();
            let text = read_file(&path);
            println!("test deser_events: parsing {}", filename);
            match ::serde_json::from_str::<EventTypes>(&text) {
                Ok(res) => {
                    // println!("{:?}",res);
                },
                Err(error) => {
                    failed = true;
                    println!("test deser_events: Error {}",error);
                }
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
    let sync_json = read_file("tests/sync.json");
    println!("test deser_sync: trying to parse sync");
    ::serde_json::from_str::<SyncReply>(&sync_json);
    println!("test deser_sync: sucessfully parsed sync!");
}
