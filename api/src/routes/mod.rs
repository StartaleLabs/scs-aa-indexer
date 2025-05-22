use axum::{extract::{Path, State}, Json};
use axum::http::StatusCode;
use crate::db::Db;
use crate::models::UserOperationRecord;

pub async fn get_user_op(
    Path(user_op_hash): Path<String>,
    State(db): State<Db>,
) -> Result<Json<UserOperationRecord>, StatusCode> {
    let user_op_hash = user_op_hash.trim();
    tracing::info!("🔍 Fetching UserOpMessage with hash: {}", user_op_hash);

    let query_result = sqlx::query_as::<_, UserOperationRecord>(
        "SELECT * FROM pm_user_operations WHERE user_op_hash = $1"
    )
    .bind(user_op_hash)
    .fetch_one(&db)
    .await;

    tracing::info!("🔍 Query result: {:?}", query_result);

    match query_result {
        Ok(record) => {
            tracing::info!("✅ Found record for hash: {}", user_op_hash);
            Ok(Json(record))
        },
        Err(sqlx::Error::RowNotFound) => {
            tracing::error!("⚠️ No record found for hash: {}", user_op_hash);
            Err(StatusCode::NOT_FOUND)
        },
        Err(e) => {
            tracing::error!("❌ DB error while fetching hash {}: {:?}", user_op_hash, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn health_check() -> (StatusCode, Json<&'static str>) {
    (StatusCode::OK, Json("OK"))
}
