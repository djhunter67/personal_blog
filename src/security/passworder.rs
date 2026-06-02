use base64::{Engine, engine::general_purpose};
use chacha20::{ChaCha20, KeyIvInit, cipher::StreamCipher};
use tracing::instrument;

use crate::security::PEPPER;

/// Encryption Logic
#[derive(Debug)]
pub struct PassWorder {
    pw: String,
}

impl PassWorder {
    /// Implementor for the password
    #[instrument(
        name = "Password Encryption",
        level = "info",
        target = "sundayLifeServices web app",
        skip(pw)
    )]
    pub fn new(pw: String) -> Self {
        PassWorder { pw }
    }

    #[instrument(
        name = "User registration attempted",
        level = "info",
        target = "sundayLifeServices web app"
    )]
    pub fn get(self) -> String {
        self.pw
    }

    /// Encrypt the password
    #[instrument(
        name = "Password Encryption",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    pub fn encrypt(self) -> Self {
        let key: [u8; 32] = *b"an example very very secret key!";
        let nonce: [u8; 12] = *b"unique nonce";

        let mut encryptor = ChaCha20::new(&key.into(), &nonce.into());

        let mut cipher_text = self.pw.into_bytes();

        encryptor.apply_keystream(&mut cipher_text);

        tracing::debug!("Encrypting");

        // let mut pw = Self::new(hex::encode(&cipher_text));

        // pw.pw.insert(16, '$');

        // pw
        Self::new(hex::encode(&cipher_text))
    }

    #[instrument(
        name = "Password Salting",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    pub fn salt(mut self) -> Self {
        let _salted = String::from("The salted deal");
        tracing::debug!("Salting");

        let random_salt: [u8; 16] = rand::random();

        // self.pw += &String::from_utf8_lossy(&random_salt);

        self.pw
            .insert_str(0, &format!("{}$", &hex::encode(random_salt)));

        // info!("The generated Salt: {}", hex::encode(random_salt));

        Self::new(self.pw)
    }

    #[instrument(
        name = "Password Peppering",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    pub fn pepper(mut self) -> Self {
        // self.pw += "the_pepper";

        let base_64_pepper = general_purpose::STANDARD.encode(PEPPER);

        self.pw += &String::from_utf8_lossy(base_64_pepper.as_bytes());
        // info!("The Peppered PW: {}", self.pw);
        Self::new(self.pw)
    }

    // #[instrument(
    //     name = "Create Hash",
    //     level = "info",
    //     target = "sundayLifeServices web app",
    //     skip(self)
    // )]
    // pub async fn create_hash(mut self) -> String {
    //     let new_hash = self.pw.clone()

    //     String::new()
    // }

    #[instrument(
        name = "Password deconstructor",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    pub fn deconstruct(&self) -> (String, String, String) {
        let (salt, hash) = self.pw.split_once('$').expect("No split delimeter found");

        match general_purpose::STANDARD.decode(
            self.pw
                .split_at(self.pw.len() - general_purpose::STANDARD.encode(PEPPER).len())
                .1,
        ) {
            Ok(pepper) => (
                String::from(salt),
                String::from(hash),
                String::from_utf8_lossy(&pepper).to_string(),
            ),
            Err(err) => {
                tracing::error!("Base64 decode failure: {err:?}");
                (
                    String::from(salt),
                    String::from(hash),
                    String::from(self.pw.split_at(self.pw.len() - PEPPER.len()).1),
                )
            }
        }
    }
}
