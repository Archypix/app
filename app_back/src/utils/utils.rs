use rand::rngs::OsRng;
use rand::RngCore;

pub fn random_token(bytes: usize) -> Vec<u8> {
    let mut auth_token = vec![0u8; bytes];
    OsRng.fill_bytes(&mut auth_token);
    auth_token
}

pub fn random_code(digits: u32) -> u32 {
    OsRng.next_u32() % 10u32.pow(digits)
}

pub fn left_pad(string: &str, char: char, target_length: usize) -> String {
    let mut res = String::new();
    for _ in 0..target_length - string.len() {
        res.push(char);
    }
    res.push_str(string);
    res
}

pub fn get_frontend_host() -> String {
    std::env::var("FRONTEND_HOST").expect("FRONTEND_HOST must be set")
}
