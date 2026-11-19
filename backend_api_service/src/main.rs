//! # Phase 4: Production-Grade REST API with Axum, Serde & SQLx SQLite
//!
//! ## Key Backend Architecture:
//! - **Axum Web Framework**: Type-safe routing, extractors (`Path`, `Query`, `Json`, `State`),
//!   and composable Tower middleware.
//! - **Serde Serialization**: Compile-time derived `Serialize` and `Deserialize` for JSON bodies.
//! - **SQLx SQLite**: Non-blocking asynchronous SQL execution, connection pooling (`SqlitePool`),
//!   and automatic schema initialization.
//! - **Structured Domain Errors**: `AppError` implements `IntoResponse`, mapping internal/validation
//!   errors directly to HTTP status codes (400, 404, 409, 500) and consistent JSON responses.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

// --- 1. Shared Application State ---

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

// --- 2. Domain Models & DTOs ---

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GenericMessageResponse {
    pub message: String,
}

// --- 3. Structured Application Errors (HTTP Mapping) ---

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        // SQLite constraint error code for UNIQUE violation is 2067 or 1555
        if let sqlx::Error::Database(ref db_err) = err
            && db_err.message().contains("UNIQUE")
        {
            return AppError::Conflict("A user with this username already exists".to_string());
        }
        AppError::DatabaseError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database failure: {e}"),
            ),
        };

        let body = Json(serde_json::json!({
            "status": status.as_u16(),
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// --- 4. Database Setup & Migrations ---

pub async fn setup_database(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Create table if it does not exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

// --- 5. Route Handlers ---

pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "healthy",
            "framework": "Axum 0.8",
            "database": "SQLite / SQLx"
        })),
    )
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let username = payload.username.trim();
    let email = payload.email.trim();

    if username.is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    let inserted_id = sqlx::query("INSERT INTO users (username, email) VALUES (?, ?)")
        .bind(username)
        .bind(email)
        .execute(&state.pool)
        .await?
        .last_insert_rowid();

    let user = User {
        id: inserted_id,
        username: username.to_string(),
        email: email.to_string(),
    };

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<User>>, AppError> {
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = pagination.offset.unwrap_or(0).max(0);

    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email FROM users ORDER BY id ASC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT id, username, email FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {id} not found")))?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let rows_affected = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("User with id {id} not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- 6. Application Router Builder ---

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/users", post(create_user).get(list_users))
        .route("/api/users/{id}", get(get_user).delete(delete_user))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "sqlite::memory:";
    let pool = setup_database(database_url).await?;
    let state = Arc::new(AppState { pool });
    let app = create_router(state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("=== Phase 4 Production REST API running on http://{addr} ===");
    println!("Endpoints:");
    println!("  GET    http://{addr}/health");
    println!("  POST   http://{addr}/api/users");
    println!("  GET    http://{addr}/api/users?limit=10&offset=0");
    println!("  GET    http://{addr}/api/users/:id");
    println!("  DELETE http://{addr}/api/users/:id");

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt; // for oneshot()

    async fn test_app() -> Router {
        let pool = setup_database("sqlite::memory:")
            .await
            .expect("Failed to initialize test SQLite memory db");
        let state = Arc::new(AppState { pool });
        create_router(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_user_crud_flow() {
        let app = test_app().await;

        // 1. Create User Alice
        let create_req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&CreateUserPayload {
                    username: "alice".to_string(),
                    email: "alice@example.com".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        // 2. Duplicate Username Conflict (409)
        let dup_req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&CreateUserPayload {
                    username: "alice".to_string(),
                    email: "alice2@example.com".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();

        let dup_res = app.clone().oneshot(dup_req).await.unwrap();
        assert_eq!(dup_res.status(), StatusCode::CONFLICT);

        // 3. Get User by ID (200)
        let get_req = Request::builder()
            .uri("/api/users/1")
            .body(axum::body::Body::empty())
            .unwrap();

        let get_res = app.clone().oneshot(get_req).await.unwrap();
        assert_eq!(get_res.status(), StatusCode::OK);

        // 4. Delete User (204)
        let del_req = Request::builder()
            .method("DELETE")
            .uri("/api/users/1")
            .body(axum::body::Body::empty())
            .unwrap();

        let del_res = app.clone().oneshot(del_req).await.unwrap();
        assert_eq!(del_res.status(), StatusCode::NO_CONTENT);

        // 5. Verify User is gone (404)
        let get_after_del = Request::builder()
            .uri("/api/users/1")
            .body(axum::body::Body::empty())
            .unwrap();

        let get_after_res = app.oneshot(get_after_del).await.unwrap();
        assert_eq!(get_after_res.status(), StatusCode::NOT_FOUND);
    }
}
