//! Initialize and return a connection to the ``MongoDb`` database.

use tracing::{info, instrument};

use crate::settings;

#[must_use = "The connection pool must be used to interact with the database"]
#[instrument(
    name = "Get Connection Pool for MongoDb",
    level = "info",
    target = "sundayLifeServices web app",
    skip(manager)
)]
/// # Result
///  - `Ok(Database)` if the connection pool was successfully created
/// # Errors
///  - `mongodb::error::Error` if the connection pool could not be created
/// # Panics
///  - If the connection pool could not be created
pub async fn establish_connection(
    manager: mongodb::Client,
) -> Result<mongodb::Database, mongodb::error::Error> {
    info!("Get mongo connection pool");
    Ok(manager.database(&settings::get().expect("setings error").mongo.db))
    // .into_inner()
    // .database(&settings.db)
    // .collection(&settings.collection)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use mongodb::{
        Collection,
        bson::{self, Bson, Document, doc},
    };
    use r2d2::ManageConnection;
    use rstest::rstest;

    use crate::{
        models::r2d2_mongodb::client_manager::MongoClientManager,
        settings::{self, Settings},
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_connects_from_uri() {
        let settings: Settings = settings::get().unwrap();

        match MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
        {
            Ok(_) => (),
            Err(err) => panic!("URI connection failure: {err:#?}"),
        }
    }

    #[rstest]
    #[ignore = "If this fails every other mongo test fails"]
    #[tokio::test]
    #[should_panic(expected = "The Database should be up")]
    async fn test_fail_to_connect() {
        let settings: Settings = settings::get().unwrap();
        let manager = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        let pool = establish_connection(manager).await;

        // Assert that a connection has been established
        assert!(
            pool.unwrap()
                .collection::<bson::Document>("test")
                .estimated_document_count()
                .await
                .is_err()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_write_to_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // let conn = pool.get().unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test");
        let collection = db.collection("test");

        // Test query
        let result = collection
            .insert_one(doc! { "name": "John Doe" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.inserted_id.ne(&Bson::Null));
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_read_from_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test_1");
        let collection = db.collection("test_1");

        // Test query
        let _ = collection.insert_one(doc! { "name": "John Dae" }).await;

        // Test query
        let result = collection
            .find_one(doc! { "name": "John Dae" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.unwrap().get_str("name").unwrap().eq("John Dae"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_can_update_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test_2");
        let collection = db.collection("test_2");

        // Test query
        let _ = collection.insert_one(doc! { "name": "John Dae" }).await;

        // Test query
        let result = collection
            .update_one(
                doc! { "name": "John Dae" },
                doc! { "$set": { "name": "John OtherDoe" } },
            )
            .await
            .unwrap();

        let changed_result = collection
            .find_one(doc! { "name": "John OtherDoe" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.modified_count.eq(&1));

        assert!(
            changed_result
                .unwrap()
                .get_str("name")
                .unwrap()
                .eq("John OtherDoe")
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_not_found_mongo() {
        let settings: Settings = settings::get().unwrap();

        let conn = MongoClientManager::from_uri(&settings.mongo.uri)
            .await
            .unwrap()
            .connect()
            .unwrap();

        // Test query the document database, mongodb
        let db = conn.database("test_3");
        let collection: Collection<Document> = db.collection("test_3");

        // insert a document
        let _ = collection
            .insert_one(doc! { "house": "180 SW 125th Ave" })
            .await;

        // Test query
        let result = collection
            .find_one(doc! { "name": "Jane Doe" })
            .await
            .unwrap();

        // Drop the database
        assert!(db.drop().await.is_ok());

        assert!(result.is_none());
    }
}
