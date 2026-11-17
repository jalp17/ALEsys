//! Inventory / LIMS API Handlers

use crate::auth::{Claims, Permission};
use crate::state::AppState;
use crate::handlers::ApiError;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub location: Option<String>,
    pub quantity: f64,
    pub unit: Option<String>,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub owner_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInventoryRequest {
    pub name: String,
    pub item_type: Option<String>,
    pub location: Option<String>,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryRequest {
    pub name: Option<String>,
    pub item_type: Option<String>,
    pub location: Option<String>,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub lot_number: Option<String>,
    pub expiry_date: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListInventoryQuery {
    pub item_type: Option<String>,
    pub search: Option<String>,
}

fn db_error(e: sqlx::Error) -> ApiError {
    ApiError { error: format!("Database error: {}", e), code: "DB_ERROR".to_string() }
}
fn forbidden() -> ApiError {
    ApiError { error: "Forbidden".to_string(), code: "FORBIDDEN".to_string() }
}
fn not_found(msg: &str) -> ApiError {
    ApiError { error: msg.to_string(), code: "NOT_FOUND".to_string() }
}

pub async fn list_inventory_handler(
    _claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ListInventoryQuery>,
) -> Result<Json<Vec<InventoryItem>>, ApiError> {
    let mut sql = String::from("SELECT id, name, item_type, location, quantity, unit, lot_number, expiry_date, owner_id, metadata, created_at, updated_at FROM inventory_items WHERE 1=1");
    let mut bind_count = 0i32;
    if q.item_type.is_some() { bind_count += 1; sql.push_str(&format!(" AND item_type = ${}", bind_count)); }
    if q.search.is_some() { bind_count += 1; sql.push_str(&format!(" AND name ILIKE ${}", bind_count)); }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query(&sql);
    if let Some(it) = &q.item_type { query = query.bind(it); }
    if let Some(s) = &q.search { query = query.bind(format!("%{}%", s)); }
    let rows = query.fetch_all(&state.db).await.map_err(db_error)?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(InventoryItem {
            id: row.get::<Uuid, _>("id").to_string(),
            name: row.get("name"),
            item_type: row.get("item_type"),
            location: row.get("location"),
            quantity: row.get("quantity"),
            unit: row.get("unit"),
            lot_number: row.get("lot_number"),
            expiry_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("expiry_date").map(|d| d.to_rfc3339()),
            owner_id: row.get("owner_id"),
            metadata: row.get("metadata"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
    }
    Ok(Json(items))
}

pub async fn create_inventory_handler(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateInventoryRequest>,
) -> Result<Json<InventoryItem>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let id = Uuid::new_v4();
    let item_type = payload.item_type.unwrap_or_else(|| "reagent".to_string());
    let quantity = payload.quantity.unwrap_or(0.0);
    let metadata = payload.metadata.unwrap_or(serde_json::json!({}));
    let expiry = payload.expiry_date.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|d| d.with_timezone(&chrono::Utc));
    let row = sqlx::query(
        r#"INSERT INTO inventory_items (id, name, item_type, location, quantity, unit, lot_number, expiry_date, owner_id, metadata)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
           RETURNING id, name, item_type, location, quantity, unit, lot_number, expiry_date, owner_id, metadata, created_at, updated_at"#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&item_type)
    .bind(&payload.location)
    .bind(quantity)
    .bind(&payload.unit)
    .bind(&payload.lot_number)
    .bind(expiry)
    .bind(&claims.sub)
    .bind(&metadata)
    .fetch_one(&state.db)
    .await
    .map_err(db_error)?;
    Ok(Json(InventoryItem {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        item_type: row.get("item_type"),
        location: row.get("location"),
        quantity: row.get("quantity"),
        unit: row.get("unit"),
        lot_number: row.get("lot_number"),
        expiry_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("expiry_date").map(|d| d.to_rfc3339()),
        owner_id: row.get("owner_id"),
        metadata: row.get("metadata"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn update_inventory_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(payload): Json<UpdateInventoryRequest>,
) -> Result<Json<InventoryItem>, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let iid = Uuid::parse_str(&item_id).map_err(|_| not_found("Invalid item id"))?;
    let mut sets = Vec::new();
    let mut bind_idx = 1i32;
    if payload.name.is_some() { sets.push(format!("name = ${}", bind_idx)); bind_idx += 1; }
    if payload.item_type.is_some() { sets.push(format!("item_type = ${}", bind_idx)); bind_idx += 1; }
    if payload.location.is_some() { sets.push(format!("location = ${}", bind_idx)); bind_idx += 1; }
    if payload.quantity.is_some() { sets.push(format!("quantity = ${}", bind_idx)); bind_idx += 1; }
    if payload.unit.is_some() { sets.push(format!("unit = ${}", bind_idx)); bind_idx += 1; }
    if payload.lot_number.is_some() { sets.push(format!("lot_number = ${}", bind_idx)); bind_idx += 1; }
    if payload.expiry_date.is_some() { sets.push(format!("expiry_date = ${}", bind_idx)); bind_idx += 1; }
    if payload.metadata.is_some() { sets.push(format!("metadata = ${}", bind_idx)); bind_idx += 1; }
    sets.push("updated_at = NOW()".to_string());
    let sql = format!(
        "UPDATE inventory_items SET {} WHERE id = ${} RETURNING id, name, item_type, location, quantity, unit, lot_number, expiry_date, owner_id, metadata, created_at, updated_at",
        sets.join(", "), bind_idx
    );
    let mut q = sqlx::query(&sql);
    if let Some(v) = &payload.name { q = q.bind(v); }
    if let Some(v) = &payload.item_type { q = q.bind(v); }
    if let Some(v) = &payload.location { q = q.bind(v); }
    if let Some(v) = payload.quantity { q = q.bind(v); }
    if let Some(v) = &payload.unit { q = q.bind(v); }
    if let Some(v) = &payload.lot_number { q = q.bind(v); }
    if let Some(v) = &payload.expiry_date {
        let d = chrono::DateTime::parse_from_rfc3339(v).ok().map(|d| d.with_timezone(&chrono::Utc));
        q = q.bind(d);
    }
    if let Some(v) = &payload.metadata { q = q.bind(v); }
    q = q.bind(iid);
    let row = q.fetch_optional(&state.db).await.map_err(db_error)?
        .ok_or_else(|| not_found("Item not found"))?;
    Ok(Json(InventoryItem {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        item_type: row.get("item_type"),
        location: row.get("location"),
        quantity: row.get("quantity"),
        unit: row.get("unit"),
        lot_number: row.get("lot_number"),
        expiry_date: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("expiry_date").map(|d| d.to_rfc3339()),
        owner_id: row.get("owner_id"),
        metadata: row.get("metadata"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
    }))
}

pub async fn delete_inventory_handler(
    claims: Claims,
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !crate::auth::has_permission(&claims.role, Permission::IngestionWrite) {
        return Err(forbidden());
    }
    let iid = Uuid::parse_str(&item_id).map_err(|_| not_found("Invalid item id"))?;
    let result = sqlx::query("DELETE FROM inventory_items WHERE id = $1").bind(iid).execute(&state.db).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(not_found("Item not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
