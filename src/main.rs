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
    id: usize,
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
        router!(request,
                (GET) (/) => {
                    let file = File::open("tasko-web/index.html").unwrap();
                    rouille::Response::from_file("text/html", file)
                },

                (GET) (/json) => {
                    let data = main.lock().unwrap();
                    rouille::Response::json(&*data)
                },

                (POST) (/newBoard) => {
                    let mut data = main.lock().unwrap();

                    let input = try_or_400!(post_input!(request, {
                        name: String,
                    }));

                    data.boards.push(Board { name: input.name, lists: Vec::new() });

                    rouille::Response::text("Success")
                },

                _ => {
                    rouille::Response::text("Not Found").with_status_code(404)
                }
        )
    });
}
