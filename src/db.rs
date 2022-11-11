use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::vec::Vec;

use crate::{
    constants::get_db_full_path,
    navigator::ConnectedDevice,
    utils::encryption::{decrypt_data, encrypt_data},
};
use rusqlite::{Connection, Result};
use std::io::prelude::*;

const CREATE_SESSION_DB: &str = "CREATE TABLE IF NOT EXISTS session_info(id INTEGER PRIMARY KEY, username TEXT NOT NULL, password TEXT NOT NULL)";
const GET_SESSION: &str = "SELECT username, password FROM  session_info LIMIT 1";
const CREATE_SESSION_RECORD: &str = "INSERT INTO session_info(username, password) VALUES (?1, ?2)";

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct User<'a> {
    pub host_name: &'a str,
    pub ip_address: &'a str,
    pub mac_address: &'a str,
    pub status: &'a str,
}

impl<'a> User<'a> {
    pub fn new(
        host_name: &'a str,
        ip_address: &'a str,
        mac_address: &'a str,
        status: &'a str,
    ) -> Self {
        User {
            host_name,
            ip_address,
            mac_address,
            status,
        }
    }

    pub fn print_user_list(ulist: Vec<User>) -> FmtResult {
        let mut output_buffer: Vec<u8> = Vec::new();
        writeln!(
            &mut output_buffer,
            "Host Name\t\tIp Address\t\tMac Address\t\tStatus"
        )
        .unwrap();
        for user in ulist {
            writeln!(
                &mut output_buffer,
                "{}\t\t{}\t\t{}\t\t{}",
                user.host_name, user.ip_address, user.mac_address, user.status
            )
            .unwrap();
        }

        print!("{}", String::from_utf8(output_buffer).unwrap());
        Ok(())
    }
}

impl<'a> From<&'a ConnectedDevice> for User<'a> {
    fn from(device: &'a ConnectedDevice) -> Self {
        User::new(
            &device.Host_name,
            &device.IP_address,
            &device.MAC,
            &device.Status,
        )
    }
}

impl SessionInfo {
    pub fn new(
        username: String,
        password: String,
    ) -> Result<(Self, Connection), Box<dyn std::error::Error>> {
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

    pub fn create(&self, connection: Rc<Connection>) -> Result<Self, Box<dyn std::error::Error>> {
        let encrypted_password = encrypt_data(&self.password);

        connection.execute(
            CREATE_SESSION_RECORD,
            &[&self.username, &encrypted_password],
        )?;
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

    pub fn retrieve(connection: Option<Rc<Connection>>) -> SessionInfo {
        let session = match SessionInfo::get(connection) {
            Some(session) => session,
            None => {
                panic!("No credentials found please try login using, 'jaac login'");
            }
        };

        session
    }
}
