use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::vec::Vec;

use crate::{
    cli::Mode,
    constants::get_db_full_path,
    navigator::ConnectedDevice,
    types::{Error, UserRecordData},
    utils::encryption::{decrypt_data, encrypt_data},
};
use rusqlite::{Connection, Result};
use std::io::prelude::*;

use crate::queries::*;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub host_name: String,
    pub ip_address: String,
    pub mac_address: String,
    pub status: String,
    id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub mode: Mode,
    id: Option<i32>,
}

pub enum UserIdentifier {
    HostName(String),
    IpAddress(String),
    MacAddress(String),
    Id(i32),
}

impl Group {
    pub fn new(name: &str, mode: Mode) -> Self {
        Group {
            name: name.to_string(),
            mode,
            id: None,
        }
    }

    pub fn sync(&self, connection: Rc<Connection>) -> Result<Self, Box<dyn std::error::Error>> {
        let found_group = Group::get(&self.name, Some(Rc::clone(&connection)));
        if let Some(group) = found_group {
            return Ok(group);
        }
        self.create(connection)?;
        Ok(self.clone())
    }

    pub fn create(&self, connection: Rc<Connection>) -> Result<Self, Box<dyn std::error::Error>> {
        connection.execute(
            CREATE_GROUP,
            &[&self.name, &(self.mode.clone() as u8).to_string()],
        )?;
        Ok(self.clone())
    }

    pub fn delete(
        name: &str,
        connection: Rc<Connection>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        connection.execute(DELETE_GROUP, &[name])?;

        Ok(())
    }

    pub fn update(
        &self,
        name: &str,
        connection: Rc<Connection>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        connection.execute(UPDATE_GROUP, &[name])?;
        Ok(())
    }

    fn get(name: &str, connection: Option<Rc<Connection>>) -> Option<Self> {
        let db_path = get_db_full_path();
        let create_connection = || {
            let conn = Connection::open(db_path).unwrap();
            return conn;
        };
        let conn = match connection {
            Some(conn) => conn,
            None => Rc::new(create_connection()),
        };

        let mut stmt = conn.prepare(SELECT_GROUP).unwrap();
        let mut rows = stmt.query([name]).unwrap();

        if let Some(row) = rows.next().unwrap() {
            let mode = match row.get::<usize, u8>(2).unwrap() {
                1 => Mode::Allow,
                _ => Mode::Block,
            };

            let group = Group {
                name: row.get::<usize, String>(1).unwrap(),
                mode,
                id: Some(row.get::<usize, i32>(0).unwrap()),
            };

            return Some(group);
        }

        return None;
    }

    pub fn retrieve(name: &str, connection: Option<Rc<Connection>>) -> Self {
        let group = match Group::get(name, connection) {
            Some(group_) => group_,
            None => {
                panic!("No credentials found please try login using, 'jaac login'");
            }
        };

        group
    }
}

impl From<ConnectedDevice> for User {
    fn from(device: ConnectedDevice) -> Self {
        let user = if let Some(mut user) =
            User::retrieve(UserIdentifier::IpAddress(device.IP_address.clone()), None)
        {
            user.status = device.Status;
            user
        } else {
            User::new(
                device.Host_name,
                device.IP_address,
                device.MAC,
                device.Status,
                None,
            )
        };
        user
    }
}

impl<'a> User {
    pub fn new(
        host_name: String,
        ip_address: String,
        mac_address: String,
        status: String,
        id: Option<i32>,
    ) -> Self {
        User {
            host_name,
            ip_address,
            mac_address,
            status,
            id,
        }
    }

    pub fn sync(&self, connection: Rc<Connection>) -> Result<Self, Box<dyn std::error::Error>> {
        let user_record = User::retrieve(
            UserIdentifier::IpAddress(self.ip_address.clone()),
            Some(Rc::clone(&connection)),
        );
        if let Some(mut user) = user_record {
            user.status = self.status.clone(); // update the status of the record from backend to the current status
            user.update(Rc::clone(&connection))?; // update the record in db since we updated the status
            return Ok(user);
        }
        self.create(Rc::clone(&connection))?;
        Ok(self.clone())
    }

    pub fn create(&self, connection: Rc<Connection>) -> Result<Self, Box<dyn std::error::Error>> {
        connection.execute(
            CREATE_USER_RECORD,
            [
                &self.host_name,
                &self.ip_address,
                &self.mac_address,
                &self.status,
            ],
        )?;

        let user_record = User::retrieve(
            UserIdentifier::MacAddress(self.mac_address.clone()),
            Some(connection),
        ).expect("user record not found, Failed to sync user record");
        Ok(user_record)
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

    fn get(
        statement: &str,
        params: &[&str; 1],
        connection: Option<Rc<Connection>>,
    ) -> Option<Self> {
        let db_path = get_db_full_path();
        let create_connection = || {
            let conn = Connection::open(db_path).unwrap();
            return conn;
        };
        let conn = match connection {
            Some(conn) => conn,
            None => Rc::new(create_connection()),
        };

        let mut stmt = conn.prepare(statement).unwrap();
        let mut rows = stmt.query(params).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let user = User {
                id: Some(row.get::<usize, i32>(0).unwrap()),
                host_name: row.get::<usize, String>(1).unwrap(),
                ip_address: row.get::<usize, String>(2).unwrap(),
                mac_address: row.get::<usize, String>(3).unwrap(),
                status: row.get::<usize, String>(4).unwrap(),
            };

            return Some(user);
        }

        return None;
    }

    pub fn retrieve(
        identifier: UserIdentifier,
        connection: Option<Rc<Connection>>,
    ) -> Option<Self> {
        let user = match identifier {
            UserIdentifier::HostName(host_name) => {
                User::get(GET_USER_BY_HOST_NAME, &[&host_name], connection)
            }
            UserIdentifier::IpAddress(ip_address) => {
                User::get(GET_USER_BY_IP, &[&ip_address], connection)
            }
            UserIdentifier::MacAddress(mac_address) => {
                User::get(GET_USER_BY_MAC, &[&mac_address], connection)
            }
            UserIdentifier::Id(id) => User::get(GET_USER_BY_ID, &[&id.to_string()], connection),
        };

        user
    }

    pub fn update(&self, connection: Rc<Connection>) -> Result<(), &str> {
        if let Some(id) = &self.id {
            connection
                .execute(
                    UPDATE_USER,
                    [
                        &self.host_name,
                        &self.ip_address,
                        &self.mac_address,
                        &self.status,
                        &id.to_string(),
                    ],
                )
                .expect("An error occured when updating user");
        } else {
            return Err("User record isn't synced");
        }
        Ok(())
    }

    pub fn get_all_users(user_records: Vec<UserRecordData>) -> Vec<User> {
        let mut users = vec![];

        for (host_name, ip_address, mac_address, status) in user_records {
            let user = User::new(host_name, ip_address, mac_address, status, None);
            users.push(user);
        }
        users
    }

    pub fn add_to_group(
        &self,
        group_id: i32,
        connection: Rc<Connection>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        connection.execute(SET_USER_GROUP, [group_id, self.id.unwrap()])?;
        Ok(())
    }

    // TODO: implement add user to block list
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

    pub fn sync(&self, connection: Rc<Connection>) -> Result<Self, Box<dyn std::error::Error>> {
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
