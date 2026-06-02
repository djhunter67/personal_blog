use mongodb::{Client, error::Error, options::ClientOptions};
use r2d2::ManageConnection;

/// Struct for managing a pool of `MongoDB` connections
/// Managed object: `mongodb::Client`
pub struct MongoClientManager(ClientOptions);

impl MongoClientManager {
    #[must_use]
    /// Create a new `MongoClientManager` from a `ClientOptions`
    /// # Panics
    /// If the `ClientOptions` are invalid
    /// # Errors
    /// If an error occurs while parsing the `ClientOptions`
    pub const fn new(options: ClientOptions) -> Self {
        Self(options)
    }

    /// Create a new `MongoClientManager` from a `MongoDB` URI
    /// # Panics
    /// If the URI is invalid
    /// # Errors
    /// If an error occurs while parsing the URI
    pub async fn from_uri(uri: &str) -> Result<Self, Error> {
        Ok(Self(ClientOptions::parse(uri).await?))
    }
}

impl ManageConnection for MongoClientManager {
    type Connection = Client;
    type Error = Error;

    fn connect(&self) -> Result<Client, Error> {
        Client::with_options(self.0.clone())
    }

    fn is_valid(&self, _client: &mut Client) -> Result<(), Error> {
        // TODO
        Ok(())
    }

    fn has_broken(&self, _client: &mut Client) -> bool {
        // TODO
        false
    }
}
