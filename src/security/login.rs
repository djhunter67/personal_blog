use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::security::passworder::PassWorder;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LoginChecker {
    #[serde(rename = "email_input")]
    email: String,
    password_hash: String,
}

impl LoginChecker {
    #[instrument(
        name = "Password Verifier",
        level = "info",
        target = "sundayLifeServices web app",
        skip(email, password_hash)
    )]
    pub fn new(email: String, password_hash: String) -> Self {
        Self {
            email,
            password_hash,
        }
    }

    #[instrument(
        name = "get the pw hash",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self)
    )]
    pub fn get_pw(&self) -> String {
        self.password_hash.clone()
    }

    #[instrument(
        name = "Password Verifier",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self, pw)
    )]
    pub fn set_pw(&mut self, pw: String) {
        // Self {
        //     email: self.email,
        //     password_hash: pw,
        // }
        self.password_hash = pw;
    }

    #[instrument(
        name = "Get the user email",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self,)
    )]
    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    #[instrument(
        name = "Password Verifier",
        level = "info",
        target = "sundayLifeServices web app",
        skip(self, user_pw)
    )]
    pub fn pw_verify(&self, user_pw: String) -> bool {
        tracing::debug!("Verifying the user entered password");

        let encrypted_pw: PassWorder = PassWorder::new(user_pw).encrypt().salt().pepper();

        let (_salt, pw, _) = encrypted_pw.deconstruct();
        tracing::debug!("The decrypted password: {pw}");

        let doc_pw = self.password_hash.clone();
        tracing::debug!("The entered in password: {doc_pw}");

        if pw.eq(&doc_pw) {
            return true;
        }

        false
    }
}
