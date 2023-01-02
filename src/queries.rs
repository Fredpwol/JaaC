pub const CREATE_SESSION_TABLE: &str = "CREATE TABLE IF NOT EXISTS session_info(id INTEGER PRIMARY KEY, username TEXT NOT NULL, password TEXT NOT NULL)";
pub const GET_SESSION: &str = "SELECT username, password FROM  session_info LIMIT 1";
pub const CREATE_SESSION_RECORD: &str =
    "INSERT INTO session_info(username, password) VALUES (?1, ?2)";
pub const CREATE_USER_TABLE: &str = "CREATE TABLE IF NOT EXISTS
                                    users (
                                        id INTEGER PRIMARY KEY, host_name TEXT NOT NULL,
                                        ip_address TEXT NOT NULL, mac_address TEXT NOT NULL,
                                        status TEXT DEFAULT 'Offline', group INTEGER FOREIGN
                                        group_id INTEGER
                                        CONSTRAINT fk_groups
                                            FOREIGN KEY (group_id)
                                            REFERENCES groups(id)
                                    )";

pub const GET_ALL_USERS: &str = "SELECT id host_name, ip_address, mac_address, status from users";
pub const GET_USER_BY_IP: &str =
    "SELECT id host_name, ip_address, mac_address, status from users where ip_address = ? LIMIT 1";

pub const GET_USER_BY_HOST_NAME: &str =
    "SELECT id host_name, ip_address, mac_address, status from users where host_name = ? LIMIT 1";
pub const GET_USER_BY_MAC: &str =
    "SELECT id host_name, ip_address, mac_address, status from users where mac_address = ? LIMIT 1";
pub const GET_USER_BY_ID: &str =
    "SELECT id host_name, ip_address, mac_address, status from users where id = ? LIMIT 1";
pub const UPDATE_USER: &str =
    "UPDATE users SET host_name=?1, ip_address=?2, mac_address?3, status=?4 WHERE id = ?5";

pub const CREATE_GROUP_TABLE: &str = "CREATE TABLE IF NOT EXISTS groups (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE , mode TEXT DEFAULT '0')";
pub const SET_USER_GROUP: &str = "UPDATE users SET group_id=? WHERE id=?";

pub const CREATE_DEFAULT_BLOCK_LIST: &str = "INSERT INTO groups(name) VALUES (default_block_list)";
pub const CREATE_GROUP: &str = "INSERT INTO groups(name, mode) VALUES (?1, ?2)";

pub const UPDATE_GROUP: &str = "UPDATE groups SET name=?1, mode=?2 WHERE name=?3";
pub const SELECT_GROUP: &str = "SELECT id, name, mode FROM groups WHERE name = ?";
pub const DELETE_GROUP: &str = "DELETE FROM groups WHERE name=?";
