use wither::{
    mongodb::{Client, Database},
    Model, WitherError,
};

use crate::{settings::APP_SETTINGS, todo::Todo};

/// Initializes the mongo database
pub async fn init_db() -> Result<Database, WitherError> {
    let db = Client::with_uri_str(&APP_SETTINGS.db_uri)
        .await?
        .database("todo-app");

    Todo::sync(&db).await?;

    Ok(db)
}
