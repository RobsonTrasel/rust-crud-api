use serde::{Serialize, Deserialize};
use postgres::{Client, NoTls, Error as PostgresError};

pub struct Database {
    client: Client,
}

impl Database {
    pub fn new(database_url: &str) -> Result<Self, PostgresError> {
        let client = Client::connect(database_url, NoTls)?;
        Ok(Self { client })
    }

    pub fn init(&mut self) -> Result<(), PostgresError> {
        self.client.batch_execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                name VARCHAR NOT NULL,
                email VARCHAR NOT NULL
            )"
        )?;
        Ok(())
    }

    pub fn create_user(&mut self, name: &str, email: &str) -> Result<(), PostgresError> {
        self.client.execute(
            "INSERT INTO users (name, email) VALUES ($1, $2)",
            &[&name, &email],
        )?;
        Ok(())
    }

    pub fn get_user(&mut self, id: i32) -> Result<User, PostgresError> {
        let row = self.client.query_one("SELECT * FROM users WHERE id = $1", &[&id])?;
        Ok(User {
            id: Some(row.get(0)),
            name: row.get(1),
            email: row.get(2),
        })
    }

    pub fn get_all_users(&mut self) -> Result<Vec<User>, PostgresError> {
        let mut users = Vec::new();
        for row in self.client.query("SELECT * FROM users", &[])? {
            users.push(User {
                id: Some(row.get(0)),
                name: row.get(1),
                email: row.get(2),
            });
        }
        Ok(users)
    }

    pub fn update_user(&mut self, id: i32, name: &str, email: &str) -> Result<(), PostgresError> {
        self.client.execute(
            "UPDATE users SET name = $1, email = $2 WHERE id = $3",
            &[&name, &email, &id],
        )?;
        Ok(())
    }

    pub fn delete_user(&mut self, id: i32) -> Result<(), PostgresError> {
        self.client.execute("DELETE FROM users WHERE id = $1", &[&id])?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
}
