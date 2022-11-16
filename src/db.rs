use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::vec::Vec;

use crate::{
    constants::get_db_full_path,
    navigator::ConnectedDevice,
    types::{Error, UserRecordData},
    utils::encryption::{decrypt_data, encrypt_data},
};
use rusqlite::{Connection, Result};
use std::io::prelude::*;

const CREATE_SESSION_TABLE: &str = "CREATE TABLE IF NOT EXISTS session_info(id INTEGER PRIMARY KEY, username TEXT NOT NULL, password TEXT NOT NULL)";
const GET_SESSION: &str = "SELECT username, password FROM  session_info LIMIT 1";
const CREATE_SESSION_RECORD: &str = "INSERT INTO session_info(username, password) VALUES (?1, ?2)";

const CREATE_USER_TABLE: &str = "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, host_name TEXT NOT NULL, ip_address TEXT NOT NULL, mac_address TEXT NOT NULL, status TEXT)";
const GET_ALL_USERS: &str = "SELECT host_name, ip_address, mac_address, status from users";
const GET_USER_BY_IP: &str =
    "SELECT host_name, ip_address, mac_address, status from users where ip_address = ?";

const CREATE_GROUP_TABLE: &str = "CREATE TABLE IF NOT EXISTS groups (id INTEGER PRIMARY KEY, name TEXT NOT NULL, mode TEXT DEFAULT '0'";

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
            "{:<25}\t{:<25}\t{:<25}\t{:<25}\n",
            "Host Name", "Ip Address", "Mac Address", "Status"
        )
        .unwrap();
        for user in ulist {
            writeln!(
                &mut output_buffer,
                "{:<25}\t{:<25}\t{:25}\t{:25}",
                user.host_name, user.ip_address, user.mac_address, user.status
            )
            .unwrap();
        }

        print!("{}", String::from_utf8(output_buffer).unwrap());
        Ok(())
    }

    pub fn get_user_data(connection: Rc<Connection>) -> Result<Vec<UserRecordData>, Error> {
        let mut stmt = connection.prepare(GET_ALL_USERS).unwrap();
        let mut rows = stmt.query([]).unwrap();

        let mut user_records = vec![];

        while let Some(row) = rows.next().unwrap() {
            let host_name = row.get::<usize, String>(0).unwrap();
            let ip_address = row.get::<usize, String>(1).unwrap();
            let mac_address = row.get::<usize, String>(2).unwrap();
            let status = row.get::<usize, String>(3).unwrap();
            let user_data = (host_name, ip_address, mac_address, status);
            user_records.push(user_data);
        }
        Ok(user_records)
    }

    pub fn get_all_users(user_records: &'a Vec<UserRecordData>) -> Vec<User<'a>> {
        let mut users = vec![];

        for (host_name, ip_address, mac_address, status) in user_records {
            let user = User::new(
                host_name.as_str(),
                ip_address.as_str(),
                mac_address.as_str(),
                status.as_str(),
            );
            users.push(user);
        }
        users
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
        conn.execute(CREATE_SESSION_TABLE, [])?;
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
