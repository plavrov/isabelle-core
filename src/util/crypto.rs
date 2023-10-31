use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;

pub fn verify_password(pw: &str, pw_hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(&pw_hash);
    Argon2::default()
        .verify_password(pw.as_bytes(), &parsed_hash.unwrap())
        .is_ok()
}

pub fn get_new_salt() -> String {
    let salt = SaltString::generate(&mut OsRng);
    return salt.to_string();
}

pub fn get_password_hash(pw: &str, salt: &str) -> String {
    let argon2 = Argon2::default();

    let saltstr = SaltString::from_b64(&salt).unwrap();
    let password_hash = argon2.hash_password(pw.as_bytes(), saltstr.as_salt());

    return password_hash.unwrap().serialize().as_str().to_string();
}

pub fn get_otp_code() -> String {
    let num = rand::thread_rng().gen_range(100000000..999999999);
    num.to_string()
}
