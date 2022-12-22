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

pub const GET_ALL_USERS: &str = "SELECT host_name, ip_address, mac_address, status from users";
pub const GET_USER_BY_IP: &str =
    "SELECT host_name, ip_address, mac_address, status from users where ip_address = ?";

pub const CREATE_GROUP_TABLE: &str = "CREATE TABLE IF NOT EXISTS groups (id INTEGER PRIMARY KEY, name TEXT NOT NULL, mode TEXT DEFAULT '0')";
pub const SET_USER_GROUP: &str = "UPDATE users SET group_id=? WHERE id=?";
