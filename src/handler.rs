use std::io::{Read, Write};
use std::net::TcpStream;
use serde_json::Result as SerdeResult;
use crate::database::{Database, User};

pub struct RequestHandler {
    database: Database,
}

impl RequestHandler {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn handle_request(&mut self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        let (status_line, content) = match self.parse_request(&request) {
            Ok((method, path)) => self.route_request(method, path, &request),
            Err(_) => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        };

        let response = format!("{}{}", status_line, content);
        stream.write_all(response.as_bytes()).unwrap();
    }

    fn parse_request<'a>(&self, request: &'a str) -> Result<(&'a str, &'a str), &str> {
        let request_line = request.lines().next().ok_or("Invalid request")?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid request");
        }
        Ok((parts[0], parts[1]))
    }

    fn route_request(&mut self, method: &str, path: &str, request: &str) -> (String, String) {
        match (method, path) {
            ("POST", "/users") => self.handle_post_request(request),
            ("GET", p) if p.starts_with("/users/") => self.handle_get_request(p),
            ("GET", "/users") => self.handle_get_all_request(),
            ("PUT", p) if p.starts_with("/users/") => self.handle_put_request(p, request),
            ("DELETE", p) if p.starts_with("/users/") => self.handle_delete_request(p),
            _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
        }
    }

    fn handle_post_request(&mut self, request: &str) -> (String, String) {
        match self.parse_user_request_body(request) {
            Ok(user) => {
                self.database.create_user(&user.name, &user.email).unwrap();
                (OK_RESPONSE.to_string(), "User created".to_string())
            }
            Err(_) => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
        }
    }

    fn handle_get_request(&mut self, path: &str) -> (String, String) {
        let id = self.extract_id(path).unwrap_or_default();
        match self.database.get_user(id.parse().unwrap()) {
            Ok(user) => (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap()),
            Err(_) => (NOT_FOUND.to_string(), "User not found".to_string()),
        }
    }

    fn handle_get_all_request(&mut self) -> (String, String) {
        match self.database.get_all_users() {
            Ok(users) => (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap()),
            Err(_) => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
        }
    }

    fn handle_put_request(&mut self, path: &str, request: &str) -> (String, String) {
        let id = self.extract_id(path).unwrap_or_default();
        match self.parse_user_request_body(request) {
            Ok(user) => {
                self.database.update_user(id.parse().unwrap(), &user.name, &user.email).unwrap();
                (OK_RESPONSE.to_string(), "User updated".to_string())
            }
            Err(_) => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
        }
    }

    fn handle_delete_request(&mut self, path: &str) -> (String, String) {
        let id = self.extract_id(path).unwrap_or_default();
        match self.database.delete_user(id.parse().unwrap()) {
            Ok(_) => (OK_RESPONSE.to_string(), "User deleted".to_string()),
            Err(_) => (NOT_FOUND.to_string(), "User not found".to_string()),
        }
    }

    fn parse_user_request_body(&self, request: &str) -> SerdeResult<User> {
        serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
    }

    fn extract_id<'a>(&self, path: &'a str) -> Option<&'a str> {
        path.split('/').nth(2)
    }
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";
