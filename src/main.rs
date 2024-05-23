mod database;
mod handler;

use crate::database::Database;
use crate::handler::RequestHandler;
use std::net::TcpListener;
use std::env;

fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut database = Database::new(&database_url).expect("Failed to connect to database");

    database.init().expect("Failed to initialize database");

    let mut handler = RequestHandler::new(database);
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server started at port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handler.handle_request(stream),
            Err(e) => println!("Error: {}", e),
        }
    }
}
