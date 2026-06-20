use crate::{
    error::{ServiceError, ServiceResult},
    models::media::{PrintSelection, PrintSelectionInput},
    utils::auth::AuthUser,
};
use actix_web::http::StatusCode;
use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Database;

const PRINTS_COLLECTION: &str = "print_selections";

pub async fn list_for_user(auth: &AuthUser, db: &Database) -> Vec<PrintSelection> {
    let coll = db.collection::<PrintSelection>(PRINTS_COLLECTION);
    let mut items = vec![];
    if let Ok(mut cursor) = coll.find(doc! {"userId": auth.id}).await {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(item) = cursor.deserialize_current() {
                items.push(item);
            }
        }
    }
    items
}

pub async fn add(
    auth: &AuthUser,
    input: PrintSelectionInput,
    db: &Database,
) -> ServiceResult<PrintSelection> {
    let record = PrintSelection {
        id: None,
        user_id: auth.id,
        item_id: input.item_id,
        source: input.source,
        title: input.title,
        thumbnail_url: input.thumbnail_url,
        media_url: input.media_url,
        created_at: DateTime::now(),
    };

    let coll = db.collection::<PrintSelection>(PRINTS_COLLECTION);
    let result = coll
        .insert_one(&record)
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(PrintSelection {
        id: result.inserted_id.as_object_id(),
        ..record
    })
}

pub async fn remove(auth: &AuthUser, selection_id: &str, db: &Database) -> ServiceResult<()> {
    let oid = ObjectId::parse_str(selection_id)
        .map_err(|_| ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_ID"))?;

    let coll = db.collection::<PrintSelection>(PRINTS_COLLECTION);
    coll
        .delete_one(doc! {"_id": oid, "userId": auth.id})
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}
