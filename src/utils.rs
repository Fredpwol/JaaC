use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm,
    Nonce, // Or `Aes128Gcm`
};
use std::process;

use super::constants::SECRET_KEY;

pub fn get_local_ip_address() -> String {
    let known_hosts = process::Command::new("hostname")
        .arg("-I")
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();

    let wifi_ip_addr = process::Command::new("awk")
        .arg(r"{print $1}")
        .stdin(known_hosts.stdout.unwrap())
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();

    let output = wifi_ip_addr.wait_with_output().unwrap();

    return String::from_utf8(output.stdout)
        .unwrap()
        .strip_suffix("\n")
        .unwrap()
        .to_string();
}

pub fn encrypt_data(data: &str) -> String {
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(SECRET_KEY.as_bytes()); // 96-bits; unique per message
    let ciphertext = cipher.encrypt(nonce, data.as_bytes().as_ref()).unwrap();
    String::from_utf8_lossy(&ciphertext)
}


pub fn decrypt_data(encrypted_data: &str) -> String {
    println!("{}", encrypted_data);
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(SECRET_KEY.as_bytes()); // 96-bits; unique per message
    let plaintext = cipher
        .decrypt(nonce, encrypted_data.as_bytes().as_ref())
        .unwrap();
    String::from_utf8_lossy(&plaintext).to_string()
}
