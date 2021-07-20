mod errors;
mod init_db;
mod settings;
mod todo;

use std::convert::TryInto;

use actix_web::{
    delete, get, patch, post, put,
    web::{Data, Json, JsonConfig, Path, Query},
    App, HttpServer,
};
use errors::TodoErrors;
use futures_util::TryStreamExt;
use init_db::init_db;
use todo::{NewTodoInput, SubstituteTodoInput, Todo, UpdateTodoInput};
use wither::{
    bson::{doc, oid::ObjectId, Document, Regex},
    mongodb::{
        options::{FindOneAndReplaceOptions, FindOneAndUpdateOptions, ReturnDocument},
        Database as MongoDatabase,
    },
    Model,
};

use crate::settings::APP_SETTINGS;

#[post("/todo")]
/// Handle the todo creation
async fn post_todo(
    db: Data<MongoDatabase>,
    input: Json<NewTodoInput>,
) -> Result<Json<Todo>, TodoErrors> {
    let mut todo: Todo = input.into();

    todo.save(&db, None).await?;

    Ok(Json(todo))
}

#[get("/todo")]
/// Handle reading all todos
async fn get_all_todos(
    db: Data<MongoDatabase>,
    search: Query<UpdateTodoInput>,
) -> Result<Json<Vec<Todo>>, TodoErrors> {
    let mut search_doc = Document::new();
    if let Some(summary) = &search.summary {
        let regex = Regex {
            pattern: summary.clone(),
            options: "i".to_string(),
        };
        search_doc.insert("summary", regex);
    }
    if let Some(description) = &search.description {
        let regex = Regex {
            pattern: description.clone(),
            options: "i".to_string(),
        };
        search_doc.insert("description", regex);
    }
    if let Some(completed) = &search.completed {
        search_doc.insert("completed", completed);
    }

    let todos = Todo::find(&db, search_doc, None)
        .await?
        .try_collect()
        .await?;

    // Return the vector
    Ok(Json(todos))
}

/// Handle getting a Todo by ID
#[get("/todo/{id}")]
async fn get_todo(
    db: Data<MongoDatabase>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, TodoErrors> {
    let oid = ObjectId::with_string(&id)?;

    let todo = Todo::find_one(&db, Some(doc! {"_id": oid}), None)
        .await?
        .ok_or(TodoErrors::TodoNotFound(id))?;

    Ok(Json(todo))
}

/// Handle updating a todo by ID (substitution)
#[put("/todo/{id}")]
async fn put_todo(
    db: Data<MongoDatabase>,
    Path(id): Path<String>,
    input: Json<SubstituteTodoInput>,
) -> Result<Json<Todo>, TodoErrors> {
    let oid = ObjectId::with_string(&id)?;

    let update_doc: Document = input.into_inner().try_into()?;

    let update_options = FindOneAndReplaceOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let todo = Todo::find_one_and_replace(&db, doc! {"_id": oid}, update_doc, Some(update_options))
        .await?
        .ok_or(TodoErrors::TodoNotFound(id))?;

    Ok(Json(todo))
}

/// Handle partial update of a todo by ID
#[patch("/todo/{id}")]
async fn patch_todo(
    db: Data<MongoDatabase>,
    Path(id): Path<String>,
    input: Json<UpdateTodoInput>,
) -> Result<Json<Todo>, TodoErrors> {
    let oid = ObjectId::with_string(&id)?;

    let update_doc: Document = input.into_inner().try_into()?;

    let update_options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    let todo = Todo::find_one_and_update(
        &db,
        doc! {"_id": oid},
        doc! {"$set": update_doc},
        update_options,
    )
    .await?
    .ok_or(TodoErrors::TodoNotFound(id))?;

    Ok(Json(todo))
}

/// Handle deletion of a todo by ID
#[delete("/todo/{id}")]
async fn delete_todo(
    db: Data<MongoDatabase>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, TodoErrors> {
    let oid = ObjectId::with_string(&id)?;

    let todo = Todo::find_one_and_delete(&db, doc! {"_id": oid}, None)
        .await?
        .ok_or(TodoErrors::TodoNotFound(id))?;

    Ok(Json(todo))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initializes the db
    let db = init_db().await.expect("Could not initialize database!");

    // custom `Json` extractor configuration
    let json_cfg = JsonConfig::default()
        // use custom error handler
        .error_handler(|err, _req| TodoErrors::InvalidJsonBody(err).into());

    let bind_address = format!("0.0.0.0:{}", &APP_SETTINGS.server_port);

    HttpServer::new(move || {
        App::new()
            .app_data(json_cfg.clone())
            // Adds db into the app context
            .data(db.clone())
            // Register all our routes
            .service(post_todo)
            .service(get_all_todos)
            .service(get_todo)
            .service(put_todo)
            .service(patch_todo)
            .service(delete_todo)
    })
    .bind(bind_address)?
    .run()
    .await
}
