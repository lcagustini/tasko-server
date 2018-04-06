#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate rocket;
extern crate rocket_contrib;

extern crate chrono;

use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use rocket::response::NamedFile;
use rocket::response::status::*;
use std::collections::HashMap;
use std::io::prelude::*;

#[derive(Deserialize)]
struct ItemJSON {
    name: String,
    due_time: Option<chrono::DateTime<chrono::Utc>>,
    note: Option<String>,

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
struct Item {
    name: String,
    due_time: Option<chrono::DateTime<chrono::Utc>>,
    note: Option<String>,
    checked: bool,
}

fn save_to_file(data: &UnwrappedBoards) {
    let file = std::fs::File::create("data.json");
    let serial = serde_json::to_string(data);

    if file.is_ok() && serial.is_ok() {
        let _ = file.unwrap().write_all(serial.unwrap().as_bytes());
    }
    else {
        println!("Failed to save changes to file");
    }
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
        .mount("/new", routes![new_board, new_list, new_item])
        .mount("/del", routes![del_board, del_list, del_item])
        .mount("/upd", routes![upd_check])
        .manage(load_from_file())
        .launch();
}

//Route "/new"
#[post("/board", format="application/json", data="<json>")]
fn new_board(json: rocket_contrib::Json<BoardJSON>, data: rocket::State<Boards>) -> NoContent {
    let mut boards = data.write().unwrap();

    boards.insert(json.into_inner().name, HashMap::new());

    save_to_file(&*boards);
    NoContent
}

#[post("/list", format="application/json", data="<json>")]
fn new_list(json: rocket_contrib::Json<ListJSON>, data: rocket::State<Boards>) -> Result<NoContent, BadRequest<()>> {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    {
        let board = match boards.get_mut(&json.board) {
            None => return Err(BadRequest(None)),
            Some(b) => b,
        };

        board.insert(json.name, HashMap::new());
    }

    save_to_file(&*boards);
    Ok(NoContent)
}

#[post("/item", format="application/json", data="<json>")]
fn new_item(json: rocket_contrib::Json<ItemJSON>, data: rocket::State<Boards>) -> Result<NoContent, BadRequest<()>> {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    {
        let board = match boards.get_mut(&json.board) {
            None => return Err(BadRequest(None)),
            Some(b) => b,
        };

        let list = match board.get_mut(&json.list) {
            None => return Err(BadRequest(None)),
            Some(l) => l,
        };

        list.insert(json.name.clone(),
            Item{ name: json.name, due_time: json.due_time, note: json.note, checked: false }
        );
    }

    save_to_file(&*boards);
    Ok(NoContent)
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
fn del_board(json: rocket_contrib::Json<BoardJSON>, data: rocket::State<Boards>) -> NoContent {
    let mut boards = data.write().unwrap();

    boards.remove(&json.into_inner().name);

    save_to_file(&*boards);
    NoContent
}

#[delete("/list", format="application/json", data="<json>")]
fn del_list(json: rocket_contrib::Json<ListJSON>, data: rocket::State<Boards>) -> Result<NoContent, BadRequest<()>> {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    {
        let board = match boards.get_mut(&json.board) {
            None => return Err(BadRequest(None)),
            Some(b) => b,
        };

        board.remove(&json.name);
    }

    save_to_file(&*boards);
    Ok(NoContent)
}

#[delete("/item", format="application/json", data="<json>")]
fn del_item(json: rocket_contrib::Json<ItemJSON>, data: rocket::State<Boards>) -> Result<NoContent, BadRequest<()>> {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    {
        let board = match boards.get_mut(&json.board) {
            None => return Err(BadRequest(None)),
            Some(b) => b,
        };

        let list = match board.get_mut(&json.list) {
            None => return Err(BadRequest(None)),
            Some(l) => l,
        };

        list.remove(&json.name);
    }

    save_to_file(&*boards);
    Ok(NoContent)
}

//Route "/upd"
#[patch("/check", format="application/json", data="<json>")]
fn upd_check(json: rocket_contrib::Json<ItemJSON>, data: rocket::State<Boards>) -> Result<NoContent, BadRequest<()>> {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    {
        let board = match boards.get_mut(&json.board) {
            None => return Err(BadRequest(None)),
            Some(b) => b,
        };

        let list = match board.get_mut(&json.list) {
            None => return Err(BadRequest(None)),
            Some(l) => l,
        };

        let item = match list.get_mut(&json.name) {
            None => return Err(BadRequest(None)),
            Some(i) => i,
        };

        item.checked = !item.checked;
    }

    save_to_file(&*boards);
    Ok(NoContent)
}
