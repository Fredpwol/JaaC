use std::process;

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

pub mod encryption {

    use aes_gcm::{
        aead::{Aead, KeyInit, OsRng},
        Aes256Gcm,
        Nonce, // Or `Aes128Gcm`
    };

    use crate::constants::SECRET_KEY;

    const KEY_LEN: usize = 32;

    fn hex_digest(data: &[u8]) -> String {
        let mut res = String::new();

        for byte in data {
            let hex_repr = format!("{:02x}", byte);
            res += &hex_repr;
        }

        res
    }

    fn hex_undigest(data: &str) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];

        for (i, b) in data.chars().step_by(2).enumerate() {
            let ms_nibb: u8 = b.to_digit(16).unwrap() as u8;
            let ls_nibb: u8 = data.chars().nth(i * 2 + 1).unwrap().to_digit(16).unwrap() as u8;

            let byte = ls_nibb | (ms_nibb << 4);
            res.push(byte);
        }

        res
    }

    fn parse_data(data: &str) -> ([u8; 32], Vec<u8>) {
        let key_data = &data[0..(KEY_LEN * 2)];
        let body_data = &data[(KEY_LEN * 2)..];

        let mut key_buffer: [u8; KEY_LEN] = [0; KEY_LEN];
        let key = self::hex_undigest(key_data);
        let body = self::hex_undigest(body_data);

        key_buffer.copy_from_slice(&key);

        (key_buffer, body)
    }

    pub fn encrypt_data(data: &str) -> String {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(SECRET_KEY.as_bytes()); // 96-bits; unique per message
        let ciphertext = cipher.encrypt(nonce, data.as_bytes().as_ref()).unwrap();

        let cipher_hex = self::hex_digest(&ciphertext);
        let key_hex = self::hex_digest(&key);

        key_hex + &cipher_hex
    }

    pub fn decrypt_data(encrypted_data: &str) -> String {
        let (key, raw_data) = self::parse_data(encrypted_data);
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(SECRET_KEY.as_bytes()); // 96-bits; unique per message
        let plaintext = cipher.decrypt(nonce, raw_data.as_ref()).unwrap();
        String::from_utf8(plaintext).unwrap()
    }
}
