extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rouille;

use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::fs::File;

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
    size: usize,
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

fn main() {
    println!("Now listening on localhost:8000");

    let main = Arc::new(Mutex::new(Main { boards: Vec::new(), size: 0 }));
    {
        let mut data = main.lock().unwrap();

        match load_from_file() {
            Some(result) => *data = result,
            None => println!("Error loading from file"),
        }
    }

    rouille::start_server("localhost:8000", move |request| {
        {
            let response = rouille::match_assets(&request, ".");
            if response.is_success() {
                return response;
            }
        }

        router!(request,
                (GET) (/debug) => {
                    let file = File::open("tasko-web/index.html").unwrap();
                    rouille::Response::from_file("text/html", file)
                },

                (GET) (/) => {
                    let file = File::open("tasko-web/interface.html").unwrap();
                    rouille::Response::from_file("text/html", file)
                },

                (GET) (/json) => {
                    let data = main.lock().unwrap();
                    rouille::Response::json(&*data)
                },

                (GET) (/load) => {
                    let mut data = main.lock().unwrap();

                    match load_from_file() {
                        Some(result) => *data = result,
                        None => println!("Error loading from file"),
                    }

                    rouille::Response::redirect_303("/")
                },

                (POST) (/newText) => {
                    let mut data = main.lock().unwrap();

                    let input = try_or_400!(post_input!(request, {
                        board: String,
                        list: String,
                        text: String,
                    }));

                    for i in 0..data.boards.len() {
                        if data.boards[i].name == input.board {
                            for j in 0..data.boards[i].lists.len() {
                                if data.boards[i].lists[j].name == input.list {
                                    data.boards[i].lists[j].items.push(Item::Text(input.text));
                                    data.size += 1;
                                    break;
                                }
                            }
                            break;
                        }
                    }

                    save_to_file(&mut*data);
                    rouille::Response::redirect_303("/")
                },

                (POST) (/newList) => {
                    let mut data = main.lock().unwrap();

                    let input = try_or_400!(post_input!(request, {
                        board: String,
                        name: String,
                    }));

                    for i in 0..data.boards.len() {
                        if data.boards[i].name == input.board {
                            data.boards[i].lists.push(List { name: input.name, items: Vec::new() });
                            break;
                        }
                    }

                    save_to_file(&mut*data);
                    rouille::Response::redirect_303("/")
                },

                (POST) (/newBoard) => {
                    let mut data = main.lock().unwrap();

                    let input = try_or_400!(post_input!(request, {
                        name: String,
                    }));

                    data.boards.push(Board { name: input.name, lists: Vec::new() });

                    save_to_file(&mut*data);
                    rouille::Response::redirect_303("/")
                },

                _ => {
                    rouille::Response::text("Not Found").with_status_code(404)
                }
        )
    });
}
