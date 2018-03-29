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

struct Main {
    boards: Arc<RwLock<Vec<Board>>>,
}

type Boards = Arc<RwLock<HashMap<String, HashMap<String, HashMap<String, Item>>>>>;

fn main() {
    rocket::ignite()
        .mount("/", routes![index, files, json])
        .mount("/new", routes![new_board])
        .manage(Main { boards: Arc::new(RwLock::new(Vec::new())) })
        .manage(Boards::new(RwLock::new(HashMap::new())))
        .launch();
}

//Route "/new"
#[derive(Deserialize)]
struct NewBoardJSON {
    name: String,
}
#[post("/board", format="application/json", data="<json>")]
fn new_board(json: rocket_contrib::Json<NewBoardJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();

    boards.insert(json.into_inner().name, HashMap::new());
    //boards.push(Board { name: json.into_inner().name, lists: Vec::new() });

    rocket::response::status::NoContent
}

#[derive(Deserialize)]
struct NewListJSON {
    name: String,
    board: String,
}
#[post("/list", format="application/json", data="<json>")]
fn new_list(json: rocket_contrib::Json<NewListJSON>, data: rocket::State<Boards>) -> rocket::response::status::NoContent {
    let mut boards = data.write().unwrap();
    let json = json.into_inner();

    let board = boards.get(&json.board);
    //boards[json.board].lists.push(List { name: json.name, items: Vec::new() });

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
