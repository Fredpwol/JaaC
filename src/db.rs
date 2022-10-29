use std::{rc::Rc};

use crate::{utils::encryption::{decrypt_data, encrypt_data}, constants::{get_db_full_path}};
use std::io::prelude::*;
use rusqlite::{Connection, Result};

const CREATE_SESSION_DB: &str = "CREATE TABLE IF NOT EXISTS session_info(id INTEGER PRIMARY KEY, username TEXT NOT NULL, password TEXT NOT NULL)";
const GET_SESSION: &str = "SELECT username, password FROM  session_info LIMIT 1";
const CREATE_SESSION_RECORD: &str = "INSERT INTO session_info(username, password) VALUES (?1, ?2)";

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub username: String,
    pub password: String,
}

impl SessionInfo {
    pub fn new(username: String, password: String) -> Result<(Self, Connection), Box<dyn std::error::Error>> {
        let db_path = get_db_full_path();
        if !db_path.exists() {
            std::fs::create_dir_all(db_path.parent().unwrap())?;
            std::fs::File::create(&db_path)?;
        }
        let conn = Connection::open(&db_path)?;
        conn.execute(CREATE_SESSION_DB, [])?;
        Ok((SessionInfo { username, password }, conn))
    }

    pub fn create_or_retrieve(
        &self,
        connection: Rc<Connection>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let found_session = SessionInfo::get(Some(Rc::clone(&connection)));
        if let Some(session) = found_session {
            return Ok(session);
        }
        self.create(connection)?;
        Ok(self.clone())
    }

    pub fn create(
        &self,
        connection: Rc<Connection>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let encrypted_password = encrypt_data(&self.password);

        connection.execute(CREATE_SESSION_RECORD, &[&self.username, &encrypted_password])?;
        Ok(self.clone())
    }

    fn get(connection: Option<Rc<Connection>>) -> Option<SessionInfo> {
        let db_path = get_db_full_path();
        let create_connection = || {
            let conn = Connection::open(db_path).unwrap();
            return conn;
        };
        let conn = match connection {
            Some(conn) => conn,
            None => Rc::new(create_connection()),
        };

        let mut stmt = conn.prepare(GET_SESSION).unwrap();
        let mut rows = stmt.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let encypted_password = row.get::<usize, String>(1).unwrap();
            let password = decrypt_data(&encypted_password);
            let sess = SessionInfo {
                username: row.get(0).unwrap(),
                password,
            };

            return Some(sess);
        }

        return None;
    }

    pub fn retrieve(connection: Option<Rc<Connection>>) -> SessionInfo{
        let session = match SessionInfo::get(connection){
            Some(session) => session,
            None => {
                panic!("No credentials found please try login using, 'jaac login'");
            }
        };

        session
    }
}
