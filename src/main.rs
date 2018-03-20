extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rouille;

use std::sync::{Arc, Mutex};
use std::fs::File;

#[derive(Serialize)]
struct List {
    id: usize,
    value: String,
}

#[derive(Serialize)]
struct Board {
    name: String,
    lists: Vec<List>,
}

#[derive(Serialize)]
struct Main {
    boards: Vec<Board>,
}

fn main() {
    println!("Now listening on localhost:8000");

    let main = Arc::new(Mutex::new(Main { boards: Vec::new() }));

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

            (GET) (/new) => {
                let mut data = main.lock().unwrap();
                let size = data.boards.len();

                data.boards.push(Board { name: "test".to_owned(), lists: vec!(List { id: 0, value: "testes".to_owned() }) });
                rouille::Response::text("Success")
            },

            _ => {
                rouille::Response::text("Not Found").with_status_code(404)
            }
        )
    });
}
