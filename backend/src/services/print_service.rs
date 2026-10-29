use crate::{
    error::{ServiceError, ServiceResult},
    models::{
        media::{
            AdminPrintOrderView, PrintCheckoutInput, PrintOrder, PrintOrderItem, PrintOrderView,
            PrintSelection, PrintSelectionInput, UpdatePrintOrder,
        },
        user::User,
    },
    utils::auth::AuthUser,
};
use actix_web::http::StatusCode;
use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Database;

const PRINTS_COLLECTION: &str = "print_selections";
const ORDERS_COLLECTION: &str = "print_orders";

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
        collection_id: input.collection_id,
        collection_kind: input.collection_kind,
        collection_title: input.collection_title,
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
    coll.delete_one(doc! {"_id": oid, "userId": auth.id})
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

fn order_item(selection: &PrintSelection) -> PrintOrderItem {
    PrintOrderItem {
        item_id: selection.item_id.clone(),
        source: selection.source.clone(),
        title: selection.title.clone(),
        thumbnail_url: selection.thumbnail_url.clone(),
        media_url: selection.media_url.clone(),
        collection_id: selection.collection_id.clone(),
        collection_kind: selection.collection_kind.clone(),
        collection_title: selection.collection_title.clone(),
    }
}

fn unit_price_cents(size: &str) -> Option<i64> {
    match size {
        "8x10" => Some(2500),
        "11x14" => Some(4500),
        "16x20" => Some(7500),
        _ => None,
    }
}

fn order_view(order: PrintOrder) -> PrintOrderView {
    PrintOrderView {
        id: order.id.map(|oid| oid.to_hex()).unwrap_or_default(),
        status: order.status,
        note: order.note,
        item_count: order.items.len(),
        created_at: order
            .created_at
            .try_to_rfc3339_string()
            .unwrap_or_else(|_| order.created_at.timestamp_millis().to_string()),
        size: order.size,
        unit_price_cents: order.unit_price_cents,
        total_cents: order.total_cents,
        payment_status: order.payment_status,
    }
}

pub async fn list_orders(auth: &AuthUser, db: &Database) -> Vec<PrintOrderView> {
    let coll = db.collection::<PrintOrder>(ORDERS_COLLECTION);
    let mut orders = vec![];
    if let Ok(mut cursor) = coll.find(doc! {"userId": auth.id}).await {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(order) = cursor.deserialize_current() {
                orders.push(order);
            }
        }
    }
    orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    orders.into_iter().map(order_view).collect()
}

fn normalize_order_status(value: &str) -> Option<&'static str> {
    match value.trim() {
        "requested" => Some("requested"),
        "fulfilled" => Some("fulfilled"),
        "cancelled" => Some("cancelled"),
        _ => None,
    }
}

fn normalize_payment_status(value: &str) -> Option<&'static str> {
    match value.trim() {
        "unpaid" => Some("unpaid"),
        "invoiced" => Some("invoiced"),
        "paid" => Some("paid"),
        _ => None,
    }
}

fn admin_order_view(
    order: PrintOrder,
    email: Option<String>,
    username: Option<String>,
) -> AdminPrintOrderView {
    let user_id = order.user_id.to_hex();
    let items = order.items.clone();
    let view = order_view(order);
    AdminPrintOrderView {
        id: view.id,
        user_id,
        email,
        username,
        status: view.status,
        note: view.note,
        item_count: view.item_count,
        items,
        created_at: view.created_at,
        size: view.size,
        unit_price_cents: view.unit_price_cents,
        total_cents: view.total_cents,
        payment_status: view.payment_status,
    }
}

async fn user_identity(user_id: &bson::oid::ObjectId, db: &Database) -> (Option<String>, Option<String>) {
    match User::find_by_id(user_id, db).await {
        Some(user) => (Some(user.email), Some(user.username)),
        None => (None, None),
    }
}

pub async fn list_admin_orders(db: &Database) -> Vec<AdminPrintOrderView> {
    let coll = db.collection::<PrintOrder>(ORDERS_COLLECTION);
    let mut orders = vec![];
    if let Ok(mut cursor) = coll.find(doc! {}).await {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(order) = cursor.deserialize_current() {
                orders.push(order);
            }
        }
    }
    orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let mut names: std::collections::HashMap<
        bson::oid::ObjectId,
        (Option<String>, Option<String>),
    > = std::collections::HashMap::new();
    let mut views = Vec::with_capacity(orders.len());
    for order in orders {
        let user_id = order.user_id;
        let (email, username) = if let Some(pair) = names.get(&user_id) {
            pair.clone()
        } else {
            let pair = user_identity(&user_id, db).await;
            names.insert(user_id, pair.clone());
            pair
        };
        views.push(admin_order_view(order, email, username));
    }
    views
}

pub async fn update_order(
    id: &str,
    input: UpdatePrintOrder,
    db: &Database,
) -> ServiceResult<AdminPrintOrderView> {
    let Some(status) = normalize_order_status(&input.status) else {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_STATUS"));
    };
    let payment = match input.payment_status.as_deref() {
        Some(value) if !value.trim().is_empty() => Some(
            normalize_payment_status(value).ok_or_else(|| {
                ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_PAYMENT_STATUS")
            })?,
        ),
        _ => None,
    };
    let oid = ObjectId::parse_str(id)
        .map_err(|_| ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_ID"))?;
    let coll = db.collection::<PrintOrder>(ORDERS_COLLECTION);
    let Some(mut order) = coll
        .find_one(doc! { "_id": oid })
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    else {
        return Err(ServiceError::new(StatusCode::NOT_FOUND, "NOT_FOUND"));
    };
    order.status = status.to_string();
    let mut set = doc! { "status": status };
    if let Some(payment) = payment {
        order.payment_status = payment.to_string();
        set.insert("paymentStatus", payment);
    }
    let result = coll
        .update_one(doc! { "_id": oid }, doc! { "$set": set })
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.matched_count == 0 {
        return Err(ServiceError::new(StatusCode::NOT_FOUND, "NOT_FOUND"));
    }
    let (email, username) = user_identity(&order.user_id, db).await;
    Ok(admin_order_view(order, email, username))
}

pub async fn checkout(
    auth: &AuthUser,
    input: PrintCheckoutInput,
    db: &Database,
) -> ServiceResult<PrintOrderView> {
    let size = input.size.trim();
    let Some(unit_price_cents) = unit_price_cents(size) else {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "UNKNOWN_SIZE"));
    };

    let selections = list_for_user(auth, db).await;
    if selections.is_empty() {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "NO_SELECTIONS"));
    }

    let note = input
        .note
        .map(|note| note.trim().to_string())
        .filter(|note| !note.is_empty())
        .map(|note| note.chars().take(500).collect());

    let record = PrintOrder {
        id: None,
        user_id: auth.id,
        status: "requested".into(),
        note,
        size: size.to_string(),
        unit_price_cents,
        total_cents: unit_price_cents * selections.len() as i64,
        payment_status: "unpaid".into(),
        items: selections.iter().map(order_item).collect(),
        created_at: DateTime::now(),
    };

    let coll = db.collection::<PrintOrder>(ORDERS_COLLECTION);
    let result = coll
        .insert_one(&record)
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(order_view(PrintOrder {
        id: result.inserted_id.as_object_id(),
        ..record
    }))
}

#[cfg(test)]
mod tests {
    use super::unit_price_cents;
    use crate::models::media::PrintOrder;
    use bson::{doc, oid::ObjectId, DateTime};

    #[test]
    fn admin_status_strings_match_print_order_fields() {
        let input: crate::models::media::UpdatePrintOrder =
            serde_json::from_str(r#"{"status":"fulfilled"}"#).unwrap();
        assert!(input.payment_status.is_none());
        assert_eq!(super::normalize_order_status(&input.status), Some("fulfilled"));
        assert_eq!(super::normalize_order_status(" cancelled "), Some("cancelled"));
        assert_eq!(super::normalize_order_status("shipped"), None);
        assert_eq!(super::normalize_payment_status("invoiced"), Some("invoiced"));
        assert_eq!(super::normalize_payment_status("refunded"), None);
    }

    #[test]
    fn fixed_sizes_are_integer_cents() {
        assert_eq!(unit_price_cents("8x10"), Some(2500));
        assert_eq!(unit_price_cents("11x14"), Some(4500));
        assert_eq!(unit_price_cents("16x20"), Some(7500));
        assert_eq!(unit_price_cents("4x6"), None);
        assert_eq!(unit_price_cents("8x10").unwrap() * 2, 5000);
    }

    #[test]
    fn old_orders_default_missing_price_fields() {
        let order: PrintOrder = bson::from_document(doc! {
            "userId": ObjectId::new(),
            "status": "requested",
            "items": [],
            "createdAt": DateTime::now(),
        })
        .unwrap();
        assert_eq!(order.size, "");
        assert_eq!(order.unit_price_cents, 0);
        assert_eq!(order.total_cents, 0);
        assert_eq!(order.payment_status, "unpaid");
    }
}
