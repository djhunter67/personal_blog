use bson::doc;
use mongodb::{
    bson,
    error::Error,
    options::{ClientOptions, DatabaseOptions},
    Client, Database,
};
use r2d2::ManageConnection;

/// Struct for managing a pool of `MongoDB` connections
/// Managed object: `mongodb::Database`
pub struct DbManager {
    client_options: ClientOptions,
    db_name: String,
    db_options: DatabaseOptions,
}

impl DbManager {
    #[must_use]
    pub fn new(client_options: ClientOptions, db_name: &str, db_options: DatabaseOptions) -> Self {
        Self {
            client_options,
            db_options,
            db_name: db_name.to_owned(),
        }
    }

    /// Create a new `DbManager` from a `MongoDB` URI
    /// # Errors
    /// If an error occurs while parsing the URI
    /// # Panics
    /// If the URI is invalid
    pub async fn from_uri(uri: &str, db_name: &str) -> Result<Self, Error> {
        Ok(Self {
            client_options: ClientOptions::parse(uri).await?,
            db_name: db_name.to_owned(),
            db_options: DatabaseOptions::default(),
        })
    }
}

impl ManageConnection for DbManager {
    type Connection = Database;
    type Error = Error;

    fn connect(&self) -> Result<Database, Error> {
        let client = Client::with_options(self.client_options.clone())?;
        let db = client.database_with_options(&self.db_name, self.db_options.clone());
        Ok(db)
    }

    fn is_valid(&self, db: &mut Database) -> Result<(), Error> {
        let _ = db.run_command(doc! {"buildInfo": 1});
        Ok(())
    }

    fn has_broken(&self, db: &mut Database) -> bool {
        self.is_valid(db).is_err()
    }
}
