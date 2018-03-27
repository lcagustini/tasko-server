#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rocket;

use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::path::{Path, PathBuf};
use rocket::response::NamedFile;

#[derive(Serialize, Deserialize)]
enum Item {
	Text(String),
}

#[derive(Serialize, Deserialize)]
struct List {
	name: String,
	items: Vec<Item>,
}

#[derive(Serialize, Deserialize)]
struct Board {
	name: String,
	lists: Vec<List>,
}

#[derive(Serialize, Deserialize)]
struct Main {
	boards: Vec<Board>,
}

fn save_to_file(data: &mut Main) {
	let mut file = std::fs::File::create("data.json").unwrap();
	let serial = serde_json::to_string(data).unwrap();

	let _ = file.write_all(serial.as_bytes());
}

fn load_from_file() -> Option<Main> {
	let mut file = match std::fs::File::open("data.json") {
		Ok(file) => file,
		Err(_) => return None,
	};

	let mut s = String::new();
	let _ = file.read_to_string(&mut s);

	match serde_json::from_str(&s) {
		Ok(result) => return Some(result),
		Err(_) => return None,
	}
}

#[get("/")]
fn index() -> std::io::Result<NamedFile> {
	NamedFile::open("tasko-web/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("tasko-web/").join(file)).ok()
}

fn main() {
	rocket::ignite().mount("/", routes![index, files]).launch();
}
