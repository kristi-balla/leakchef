use crusty::LiveFeedCrypto;
use fake::{
    faker::internet::en::{FreeEmail, Password},
    Fake,
};
use lib_client::{HashedAndSaltedCredentials, PlainIdentifierPasswordPair};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

fn main() {
    println!("Hello, world!");

    let mut identities: HashMap<String, PlainIdentifierPasswordPair> =
        HashMap::with_capacity(500_000);

    for _ in 0..500_000 {
        let (hashed_and_salted_credentials, plain_identifier_password_pair) = generate();
        let creds = hashed_and_salted_credentials.to_string();
        identities.insert(creds, plain_identifier_password_pair);
    }

    let file = File::create("identities.json").unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &identities).unwrap();
    writer.flush().unwrap();
}

pub fn generate() -> (HashedAndSaltedCredentials, PlainIdentifierPasswordPair) {
    let email = FreeEmail().fake::<String>();
    let (username, _domain) = email
        .rsplit_once('@')
        .expect("Random email must contain '@'");
    let password = Password(8..16).fake::<String>();

    let domain = "hotmail.com";

    let email = format!("{username}@{domain}");
    println!("Here is the generated email: {email}");

    let plain_identifier_password_pair = PlainIdentifierPasswordPair {
        identifier: email.clone(),
        password: password.clone(),
    };

    let crypto = LiveFeedCrypto::default();

    let (id_hash, dt_enc) = crypto
        .get_salted_idhash_and_dtenc(&email, &password, b"ZZhUc2b")
        .unwrap();

    let hashed_and_salted_credentials = HashedAndSaltedCredentials { id_hash, dt_enc };
    (
        hashed_and_salted_credentials,
        plain_identifier_password_pair,
    )
}
