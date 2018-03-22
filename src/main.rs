extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rouille;

use std::sync::{Arc, Mutex};
use std::fs::File;

#[derive(Serialize)]
enum Item {
    Text(String),
}

#[derive(Serialize)]
struct List {
    name: String,
    items: Vec<Item>,
}

#[derive(Serialize)]
struct Board {
    name: String,
    lists: Vec<List>,
}

#[derive(Serialize)]
struct Main {
    boards: Vec<Board>,
    size: usize,
}

fn main() {
    println!("Now listening on localhost:8000");

    let main = Arc::new(Mutex::new(Main { boards: Vec::new(), size: 0 }));

    rouille::start_server("localhost:8000", move |request| {
        {
            let response = rouille::match_assets(&request, ".");
            if response.is_success() {
                return response;
            }
        }

        router!(request,
                (GET) (/) => {
                    let file = File::open("tasko-web/index.html").unwrap();
                    rouille::Response::from_file("text/html", file)
                },

                (GET) (/interface) => {
                    let file = File::open("tasko-web/interface.html").unwrap();
                    rouille::Response::from_file("text/html", file)
                },

                (GET) (/json) => {
                    let data = main.lock().unwrap();
                    rouille::Response::json(&*data)
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

                    rouille::Response::redirect_303("/")
                },

                (POST) (/newBoard) => {
                    let mut data = main.lock().unwrap();

                    let input = try_or_400!(post_input!(request, {
                        name: String,
                    }));

                    data.boards.push(Board { name: input.name, lists: Vec::new() });

                    rouille::Response::redirect_303("/")
                },

                _ => {
                    rouille::Response::text("Not Found").with_status_code(404)
                }
        )
    });
}
