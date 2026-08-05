use redis::Commands;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{models::mongo, security::passworder};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Users {
    email: String,
    password_hash: String,
    password_salt: String,
}

impl Users {
    pub fn new(email: String, password_hash: String, password_salt: String) -> Self {
        Self {
            email,
            password_hash,
            password_salt,
        }
    }

    /// Get the email field of the Struct ``Users``
    #[instrument(
        name = "Get the user's password",
        level = "info",
        target = "Personnel",
        skip(self)
    )]
    #[must_use = "Get the email field of the Struct ``Users``"]
    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    /// Set the email field of the Struct ``Users``
    #[instrument(
        name = "Set the user's email",
        level = "info",
        target = "Personnel",
        skip(self, new_email)
    )]
    pub fn set_email(&mut self, new_email: &str) {
        self.email = String::from(new_email)
    }

    /// Get the user's password
    #[instrument(
        name = "Get the user's password",
        level = "info",
        target = "Personnel",
        skip(self)
    )]
    pub fn get_pw(&self) -> String {
        self.password_hash.clone()
    }

    /// Set the user's password
    #[instrument(
        name = "Set the user's password",
        level = "info",
        target = "Personnel",
        skip(self, new_pw)
    )]
    pub fn set_pw(&mut self, new_pw: &str) {
        let new_pw_hash = passworder::PassWorder::new(new_pw)
            .encrypt()
            .salt()
            .pepper();
        self.password_hash = new_pw_hash.to_string();
    }

    /// Take in a plain-text unencrypted password to check against
    /// It is expected the the ``user_to_check` is the ``Users`` pulled directly from the database
    #[instrument(
        name = "Ensure the passwords are equivalent",
        level = "info",
        target = "Personnel",
        skip(self, db_user)
    )]
    fn compare_pw(&self, db_user: &Self) -> bool {
        tracing::debug!("The user passed in for the password comparison: {db_user:#?}");
        // let pw: (_, String, _) = PassWorder::new(db_user.get_pw()).deconstruct();

        let encrypted_pw = passworder::PassWorder::new(&self.get_pw())
            .encrypt()
            .salt()
            .get();
        // .pepper()

        // {
        //     // Troubleshooting block
        //     let (salt, pw, _pepper) = passworder::PassWorder::new(&self.get_pw())
        //         .encrypt()
        //         .salt()
        //         .pepper()
        //         .deconstruct();

        //     use base64::Engine;
        //     let base_64_pepper = base64::engine::general_purpose::STANDARD.encode(*b"the_pepperer");

        //     let pepper = String::from_utf8_lossy(base_64_pepper.as_bytes());

        //     tracing::error!("\nPWD: {pw}\nSLT: {salt}\nPEP: {pepper}");
        // }

        // let pw = db_user.get_pw();
        tracing::info!(
            "The pw's match: {} -> \npassed_in: {}\ndb_user: {}",
            encrypted_pw == db_user.password_hash,
            encrypted_pw,
            db_user.password_hash
        );

        // self.password_hash.eq(&db_user.password_hash)
        encrypted_pw.eq(&db_user.password_hash)
    }

    /// # Errors
    ///
    ///   - This will return an `anyhow` error if the connection to the database cannot be established
    #[instrument(
        name = "Password Verifier",
        level = "info",
        target = "User Login Attempt",
        skip(self, redis_conn, mongo_client)
    )]
    pub async fn pw_verify(
        &self,
        mongo_client: &mongodb::Client,
        redis_conn: &mut r2d2::PooledConnection<redis::Client>,
    ) -> anyhow::Result<bool> {
        tracing::debug!("Verifying the user entered password");

        // let encrypted_pw: PassWorder = PassWorder::new(user_pw).encrypt().salt().pepper();

        // let (_salt, pw, _) = encrypted_pw.deconstruct();
        // tracing::debug!("The decrypted password: {pw}");

        let cache_key = format!("user:auth:{}", self.email);
        tracing::warn!("The cache key to use to get the email to verify the user: {cache_key}");

        // let mut redis_conn = match redis_conf::establish_connection(redis_client) {
        //     Ok(conn) => conn,
        //     Err(err) => {
        //         tracing::error!("Unable to connect to the cache-layer: {err:?}");
        //         return Err(anyhow::Error::msg("Unable to connect to the cache-layer"));
        //     }
        // };
        // Get the user's key from when the user registered
        let user_oid: String = match redis_conn.get(cache_key) {
            Ok(cached_user) => cached_user,
            Err(err) => {
                tracing::warn!("No registration keys detected: {err}");
                String::new()
            }
        };

        tracing::warn!("The user_oid found: {user_oid:#?}");

        let oid: mongodb::bson::oid::ObjectId =
            serde_json::from_str::<mongodb::bson::oid::ObjectId>(&user_oid)?;

        tracing::warn!("The oid found: {oid}");

        // Compare the password saved and the password entered
        let filter = mongodb::bson::doc! {
            "_id": oid,
        };

        let mongo_conn = mongo::establish_connection(mongo_client).await?;
        let user: Self = mongo_conn
            .collection::<Self>("Users")
            .find_one(filter)
            .await?
            .unwrap_or_default();

        tracing::warn!("The user returned: {user:#?}");

        Ok(self.compare_pw(&user))
    }
}

#[cfg(test)]
mod test {}
