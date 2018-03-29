#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rocket;
extern crate rocket_contrib;

use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use rocket::response::NamedFile;
use std::collections::HashMap;
use std::io::prelude::*;

#[derive(Deserialize)]
struct TextJSON {
    name: String,
    list: String,
    board: String,
}
#[derive(Deserialize)]
struct ListJSON {
    name: String,
    board: String,
}
#[derive(Deserialize)]
struct BoardJSON {
    name: String,
}

type UnwrappedBoards = HashMap<String, HashMap<String, HashMap<String, Item>>>;
type Boards = Arc<RwLock<UnwrappedBoards>>;
#[derive(Serialize, Deserialize)]
enum Item {
    Text(String),
}

fn save_to_file(data: &UnwrappedBoards) {
    let mut file = std::fs::File::create("data.json").unwrap();
    let serial = serde_json::to_string(data).unwrap();

    let _ = file.write_all(serial.as_bytes());
}

fn load_from_file() -> Boards {
    let mut file = match std::fs::File::open("data.json") {
        Ok(file) => file,
        Err(_) => return Arc::new(RwLock::new(UnwrappedBoards::new())),
    };

    let mut s = String::new();
    let _ = file.read_to_string(&mut s);

    match serde_json::from_str(&s) {
        Ok(result) => return Arc::new(RwLock::new(result)),
        Err(_) => return Arc::new(RwLock::new(UnwrappedBoards::new())),
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, files, json])
        .mount("/new", routes![new_board, new_list, new_text])
        .mount("/del", routes![del_board, del_list, del_text])
        .manage(load_from_file())
        .launch();
}

//Route "/new"
#[post("/board", format="application/json", data="<json>")]
fn new_board(json: rocket_contrib::Json<BoardJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();

    boards.insert(json.into_inner().name, HashMap::new());

    save_to_file(&*boards);
    rocket::response::status::NoContent
}

#[post("/list", format="application/json", data="<json>")]
fn new_list(json: rocket_contrib::Json<ListJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    boards.get_mut(&json.board).unwrap().insert(json.name, HashMap::new());

    save_to_file(&*boards);
    rocket::response::status::NoContent
}

#[post("/text", format="application/json", data="<json>")]
fn new_text(json: rocket_contrib::Json<TextJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    boards.get_mut(&json.board).unwrap().get_mut(&json.list).unwrap().insert(json.name.clone(), Item::Text(json.name));

    save_to_file(&*boards);
    rocket::response::status::NoContent
}

//Route "/"
#[get("/")]
fn index() -> std::io::Result<NamedFile> {
    NamedFile::open("tasko-web/index.html")
}

#[get("/json")]
fn json(data: rocket::State<Boards>) -> rocket::Response {
    let boards = data.read().unwrap();

    let mut response = rocket::Response::new();
    response.set_status(rocket::http::Status::Ok);
    response.set_header(rocket::http::ContentType::JSON);

    let json = serde_json::to_string(&*boards).unwrap();
    response.set_sized_body(std::io::Cursor::new(json));

    response
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("tasko-web/").join(file)).ok()
}

//Route "/del"
#[delete("/board", format="application/json", data="<json>")]
fn del_board(json: rocket_contrib::Json<BoardJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();

    boards.remove(&json.into_inner().name);

    save_to_file(&*boards);
    rocket::response::status::NoContent
}

#[delete("/list", format="application/json", data="<json>")]
fn del_list(json: rocket_contrib::Json<ListJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    boards.get_mut(&json.board).unwrap().remove(&json.name);

    save_to_file(&*boards);
    rocket::response::status::NoContent
}

#[delete("/text", format="application/json", data="<json>")]
fn del_text(json: rocket_contrib::Json<TextJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    boards.get_mut(&json.board).unwrap().get_mut(&json.list).unwrap().remove(&json.name);

    save_to_file(&*boards);
    rocket::response::status::NoContent
}
