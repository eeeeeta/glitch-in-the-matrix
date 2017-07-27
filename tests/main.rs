extern crate glitch_in_the_matrix as matrix_api;
extern crate serde_json;

use matrix_api::types::{Event,BasicEvent};
use std::fs;
use std::io;

fn read_file(file: &str) -> String {
    let mut text = String::new();
    let mut f = fs::File::open(&file)
        .expect(&format!("File not found: '{}'!",file));
    io::Read::read_to_string(&mut f,&mut text).expect("something went wrong reading the file");
    text

}

enum Events {
    BasicEvent(BasicEvent),
    Event(Event),
}
#[test]
fn deser_events() {
    let paths = fs::read_dir("tests/event-examples").unwrap();
    for path in paths {
        let path = path.unwrap();
        let filename = path.file_name();
        let filename = filename.to_str().unwrap();
        let path = path.path();
        let path = path.to_str().unwrap();
        let text = read_file(&path);
        println!("{}:", filename);
        let parts = filename.split(".").collect::<Vec<&str>>().len();
        if parts == 2 {
            match ::serde_json::from_str::<BasicEvent>(&text) {
                Ok(res) => {
                    println!("{:?}",res);
                },
                Err(error) => {
                    println!("Error {}",error);
                }
            }
        } else {
            if let Err(error) = ::serde_json::from_str::<Event>(&text) {
                println!("Error {}",error);
            }
        };

    }
}
