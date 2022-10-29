use std::path::PathBuf;

use dirs::home_dir;

pub const BASE_URL: &str = "http://jiofi.local.html/";
pub const DB_PATH: &str = ".jaac/db/app.db";


pub const SECRET_KEY: &str = "A@FD81##0$p0";



pub fn get_db_full_path() -> PathBuf {
    let path = home_dir().unwrap().join(DB_PATH);
    path
}
