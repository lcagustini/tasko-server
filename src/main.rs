extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rouille;

#[derive(Serialize)]
struct Item {
    id: usize,
    value: String,
}

#[derive(Serialize)]
struct Main {
    mut items: Vec<Item>
}

fn main() {
    println!("Now listening on localhost:8000");

    let main = Main { items: Vec::new() };

    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (GET) (/) => {
                main.items.push(Item { id: main.items.len(), value: "test".to_owned() });
                rouille::Response::json(&main)
            },

            _ => {
                rouille::Response::text("Not Found").with_status_code(404)
            }
        )
    });
}
