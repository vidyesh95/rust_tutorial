//! # Phase 4: Production-Grade REST API with Axum, Serde, SQLx & OpenAPI / Swagger
//!
//! ## Key Backend Architecture:
//! - **Axum Web Framework**: Type-safe routing, extractors (`Path`, `Query`, `Json`, `State`),
//!   custom auth extractors (`AuthUser`), and composable Tower middleware.
//! - **Serde Serialization**: Compile-time derived `Serialize` and `Deserialize` for JSON bodies.
//! - **SQLx SQLite**: Non-blocking asynchronous SQL execution, connection pooling (`SqlitePool`),
//!   foreign key enforcement, and atomic multi-table transactions (`pool.begin()`).
//! - **OpenAPI & Swagger UI (utoipa)**: Interactive documentation served directly at `/swagger-ui`
//!   and machine-readable specification served at `/api-docs/openapi.json`.
//! - **Structured Domain Errors**: `AppError` implements `IntoResponse`, mapping internal/validation/auth
//!   errors directly to HTTP status codes (400, 401, 404, 409, 500) and consistent JSON responses.

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

// --- 1. Shared Application State ---

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

// --- 2. Domain Models & DTOs ---

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct User {
    /// Unique database identifier
    #[schema(example = 1)]
    pub id: i64,
    /// Unique username of the user
    #[schema(example = "alice")]
    pub username: String,
    /// Primary email address
    #[schema(example = "alice@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserPayload {
    /// Desired username (must be unique)
    #[schema(example = "bob")]
    pub username: String,
    /// User's contact email
    #[schema(example = "bob@example.com")]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct Task {
    /// Unique task identifier
    #[schema(example = 1)]
    pub id: i64,
    /// Foreign key referencing users(id)
    #[schema(example = 1)]
    pub user_id: i64,
    /// Title of the task
    #[schema(example = "Complete Rust tutorial")]
    pub title: String,
    /// Optional task description
    #[schema(example = "Understand SQLx transactions and Axum extractors")]
    pub description: Option<String>,
    /// Completion status flag
    #[schema(example = false)]
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTaskPayload {
    /// Title of the task
    #[schema(example = "Implement bearer authentication")]
    pub title: String,
    /// Detailed description
    #[schema(example = "Add AuthUser extractor checking Authorization header")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserWithTasksPayload {
    /// Desired username
    #[schema(example = "charlie")]
    pub username: String,
    /// User contact email
    #[schema(example = "charlie@example.com")]
    pub email: String,
    /// List of initial tasks created atomically in the same database transaction
    #[schema(example = json!(["Setup database", "Write integration tests"]))]
    pub initial_tasks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserWithTasks {
    pub user: User,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    /// Maximum number of records to return (1-100)
    #[schema(example = 20)]
    pub limit: Option<i64>,
    /// Number of records to skip
    #[schema(example = 0)]
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// HTTP status code
    #[schema(example = 404)]
    pub status: u16,
    /// Detailed error message
    #[schema(example = "User with id 1 not found")]
    pub error: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = "healthy")]
    pub status: String,
    #[schema(example = "Axum 0.8")]
    pub framework: String,
    #[schema(example = "SQLite / SQLx")]
    pub database: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminStatsResponse {
    #[schema(example = 42)]
    pub total_users: i64,
    #[schema(example = 128)]
    pub total_tasks: i64,
    #[schema(example = "Authenticated as admin")]
    pub message: String,
}

// --- 3. Authentication Extractor (Backend Auth Pattern) ---

/// Extractor that verifies the HTTP `Authorization: Bearer <token>` header.
/// In production, this would decode and verify a signed JWT (e.g. using `jsonwebtoken`).
/// For this educational backend, `secret-token-123` represents valid admin credentials.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub role: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header["Bearer ".len()..];
                if token == "secret-token-123" {
                    Ok(AuthUser {
                        user_id: 1,
                        role: "admin".to_string(),
                    })
                } else {
                    Err(AppError::Unauthorized(
                        "Invalid bearer token provided".to_string(),
                    ))
                }
            }
            _ => Err(AppError::Unauthorized(
                "Missing or invalid Authorization header. Expected: 'Bearer secret-token-123'"
                    .to_string(),
            )),
        }
    }
}

// --- 4. Structured Application Errors (HTTP Mapping) ---

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
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
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database failure: {e}"),
            ),
        };

        let body = Json(ErrorResponse {
            status: status.as_u16(),
            error: error_message,
        });

        (status, body).into_response()
    }
}

// --- 5. Database Setup & Foreign Key Migrations ---

pub async fn setup_database(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            completed BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

// --- 6. Route Handlers with OpenAPI Annotations ---

/// Service health check
#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is operational", body = HealthResponse)
    )
)]
pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy".to_string(),
            framework: "Axum 0.8".to_string(),
            database: "SQLite / SQLx (Foreign Keys Enabled)".to_string(),
        }),
    )
}

/// Create a new single user
#[utoipa::path(
    post,
    path = "/api/users",
    tag = "Users",
    request_body = CreateUserPayload,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Username conflict", body = ErrorResponse),
        (status = 500, description = "Internal database error", body = ErrorResponse)
    )
)]
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

/// Create a user along with multiple initial tasks atomically in a single SQL transaction.
/// If any task insertion fails or is invalid, the entire transaction rolls back!
#[utoipa::path(
    post,
    path = "/api/users/atomic",
    tag = "Transactions",
    request_body = CreateUserWithTasksPayload,
    responses(
        (status = 201, description = "User and tasks created atomically", body = UserWithTasks),
        (status = 400, description = "Validation failure (triggers automatic rollback)", body = ErrorResponse),
        (status = 409, description = "Username conflict", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    )
)]
pub async fn create_user_with_tasks_atomic(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserWithTasksPayload>,
) -> Result<(StatusCode, Json<UserWithTasks>), AppError> {
    let username = payload.username.trim();
    let email = payload.email.trim();

    if username.is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    // Begin an atomic transaction:
    let mut tx = state.pool.begin().await?;

    // Step 1: Insert user inside the transaction
    let user_id = sqlx::query("INSERT INTO users (username, email) VALUES (?, ?)")
        .bind(username)
        .bind(email)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

    let mut tasks = Vec::new();

    // Step 2: Insert each task inside the transaction
    for task_title in &payload.initial_tasks {
        let trimmed_title = task_title.trim();
        if trimmed_title.is_empty() {
            // Early return drops `tx` without `commit()`, automatically rolling back the inserted user!
            return Err(AppError::BadRequest(
                "Task title cannot be empty during atomic creation".to_string(),
            ));
        }

        let task_id =
            sqlx::query("INSERT INTO tasks (user_id, title, completed) VALUES (?, ?, ?)")
                .bind(user_id)
                .bind(trimmed_title)
                .bind(false)
                .execute(&mut *tx)
                .await?
                .last_insert_rowid();

        tasks.push(Task {
            id: task_id,
            user_id,
            title: trimmed_title.to_string(),
            description: None,
            completed: false,
        });
    }

    // Step 3: Commit all changes atomically
    tx.commit().await?;

    let user = User {
        id: user_id,
        username: username.to_string(),
        email: email.to_string(),
    };

    Ok((StatusCode::CREATED, Json(UserWithTasks { user, tasks })))
}

/// List users with optional limit and offset pagination
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "Users",
    params(
        ("limit" = Option<i64>, Query, description = "Max number of items to return"),
        ("offset" = Option<i64>, Query, description = "Number of items to skip")
    ),
    responses(
        (status = 200, description = "List of users", body = [User]),
        (status = 500, description = "Database error", body = ErrorResponse)
    )
)]
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

/// Retrieve a user by their unique database ID
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "User unique ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
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

/// Delete a user by their unique database ID. Cascades to all tasks owned by the user!
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "User unique ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
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

/// Create a task for a specific user (verifies foreign key relationship)
#[utoipa::path(
    post,
    path = "/api/users/{id}/tasks",
    tag = "Tasks",
    params(
        ("id" = i64, Path, description = "User ID who owns this task")
    ),
    request_body = CreateTaskPayload,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    )
)]
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    Json(payload): Json<CreateTaskPayload>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let title = payload.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("Task title cannot be empty".to_string()));
    }

    // Verify user exists first to provide a clean 404
    let user_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?
        > 0;

    if !user_exists {
        return Err(AppError::NotFound(format!("User with id {user_id} not found")));
    }

    let inserted_id = sqlx::query(
        "INSERT INTO tasks (user_id, title, description, completed) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(title)
    .bind(payload.description.as_deref())
    .bind(false)
    .execute(&state.pool)
    .await?
    .last_insert_rowid();

    let task = Task {
        id: inserted_id,
        user_id,
        title: title.to_string(),
        description: payload.description,
        completed: false,
    };

    Ok((StatusCode::CREATED, Json(task)))
}

/// List all tasks assigned to a specific user
#[utoipa::path(
    get,
    path = "/api/users/{id}/tasks",
    tag = "Tasks",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "List of user tasks", body = [Task]),
        (status = 404, description = "User not found", body = ErrorResponse)
    )
)]
pub async fn list_user_tasks(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<Vec<Task>>, AppError> {
    // Verify user exists
    let user_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await?
        > 0;

    if !user_exists {
        return Err(AppError::NotFound(format!("User with id {user_id} not found")));
    }

    let tasks = sqlx::query_as::<_, Task>(
        "SELECT id, user_id, title, description, completed FROM tasks WHERE user_id = ? ORDER BY id ASC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tasks))
}

/// Protected admin endpoint: requires `Authorization: Bearer secret-token-123` header.
#[utoipa::path(
    get,
    path = "/api/admin/stats",
    tag = "Auth",
    responses(
        (status = 200, description = "System metrics for authorized admins", body = AdminStatsResponse),
        (status = 401, description = "Unauthorized: missing or invalid Bearer token", body = ErrorResponse)
    )
)]
pub async fn protected_admin_stats(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<Json<AdminStatsResponse>, AppError> {
    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await?;

    let total_tasks = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks")
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(AdminStatsResponse {
        total_users,
        total_tasks,
        message: "Authenticated successfully with Bearer token".to_string(),
    }))
}

// --- 7. OpenAPI Documentation Specification ---

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        create_user,
        create_user_with_tasks_atomic,
        list_users,
        get_user,
        delete_user,
        create_task,
        list_user_tasks,
        protected_admin_stats,
    ),
    components(
        schemas(
            User,
            CreateUserPayload,
            CreateUserWithTasksPayload,
            UserWithTasks,
            Task,
            CreateTaskPayload,
            HealthResponse,
            ErrorResponse,
            AdminStatsResponse
        )
    ),
    tags(
        (name = "System", description = "System and health check endpoints"),
        (name = "Users", description = "User management and CRUD endpoints"),
        (name = "Tasks", description = "Task management and foreign key relationships"),
        (name = "Transactions", description = "Atomic multi-table SQL transaction operations"),
        (name = "Auth", description = "Protected endpoints requiring HTTP Bearer authentication")
    ),
    info(
        title = "Rust Tutorial Backend API",
        version = "1.0.0",
        description = "Production-grade asynchronous REST API built with Axum, SQLx SQLite, atomic transactions, bearer auth, and Utoipa OpenAPI documentation."
    )
)]
pub struct ApiDoc;

// --- 8. Application Router Builder ---

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_check))
        .route("/api/users", post(create_user).get(list_users))
        .route("/api/users/atomic", post(create_user_with_tasks_atomic))
        .route(
            "/api/users/{id}",
            get(get_user).delete(delete_user),
        )
        .route(
            "/api/users/{id}/tasks",
            post(create_task).get(list_user_tasks),
        )
        .route("/api/admin/stats", get(protected_admin_stats))
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
    println!("Interactive Documentation:");
    println!("  Swagger UI:   http://{addr}/swagger-ui");
    println!("  OpenAPI JSON: http://{addr}/api-docs/openapi.json");
    println!("\nPublic Endpoints:");
    println!("  GET    http://{addr}/health");
    println!("  POST   http://{addr}/api/users");
    println!("  POST   http://{addr}/api/users/atomic (Transaction: user + tasks)");
    println!("  GET    http://{addr}/api/users?limit=10&offset=0");
    println!("  GET    http://{addr}/api/users/:id");
    println!("  DELETE http://{addr}/api/users/:id");
    println!("  POST   http://{addr}/api/users/:id/tasks");
    println!("  GET    http://{addr}/api/users/:id/tasks");
    println!("\nProtected Endpoints (Requires 'Authorization: Bearer secret-token-123'):");
    println!("  GET    http://{addr}/api/admin/stats");

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let pool = setup_database("sqlite::memory:")
            .await
            .expect("Failed to initialize test SQLite memory db");
        let state = Arc::new(AppState { pool });
        create_router(state)
    }

    #[tokio::test]
    async fn test_openapi_and_swagger_endpoints() {
        let app = test_app().await;

        let req = Request::builder()
            .uri("/api-docs/openapi.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let swagger_req = Request::builder()
            .uri("/swagger-ui/")
            .body(axum::body::Body::empty())
            .unwrap();

        let swagger_res = app.oneshot(swagger_req).await.unwrap();
        assert_eq!(swagger_res.status(), StatusCode::OK);
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

    #[tokio::test]
    async fn test_atomic_transaction_success() {
        let app = test_app().await;

        let payload = CreateUserWithTasksPayload {
            username: "charlie".to_string(),
            email: "charlie@example.com".to_string(),
            initial_tasks: vec![
                "Setup database migrations".to_string(),
                "Implement transactions".to_string(),
            ],
        };

        let req = Request::builder()
            .method("POST")
            .uri("/api/users/atomic")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&payload).unwrap(),
            ))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        let body_bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let user_with_tasks: UserWithTasks = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(user_with_tasks.user.username, "charlie");
        assert_eq!(user_with_tasks.tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_atomic_transaction_rollback_on_invalid_task() {
        let app = test_app().await;

        // One task title is empty, which must trigger Bad Request and roll back user insertion
        let payload = CreateUserWithTasksPayload {
            username: "dave_rollback".to_string(),
            email: "dave@example.com".to_string(),
            initial_tasks: vec![
                "Valid task 1".to_string(),
                "   ".to_string(), // INVALID EMPTY TITLE!
            ],
        };

        let req = Request::builder()
            .method("POST")
            .uri("/api/users/atomic")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&payload).unwrap(),
            ))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // Verify that dave_rollback was NOT inserted (proving atomicity & rollback)
        let check_req = Request::builder()
            .uri("/api/users?limit=100")
            .body(axum::body::Body::empty())
            .unwrap();

        let check_res = app.oneshot(check_req).await.unwrap();
        let body_bytes = axum::body::to_bytes(check_res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let users: Vec<User> = serde_json::from_slice(&body_bytes).unwrap();
        assert!(
            users.iter().all(|u| u.username != "dave_rollback"),
            "User should have been rolled back!"
        );
    }

    #[tokio::test]
    async fn test_tasks_crud_and_foreign_key_cascade() {
        let app = test_app().await;

        // 1. Create a user
        let user_req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&CreateUserPayload {
                    username: "eve".to_string(),
                    email: "eve@example.com".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        let user_res = app.clone().oneshot(user_req).await.unwrap();
        assert_eq!(user_res.status(), StatusCode::CREATED);
        let user_bytes = axum::body::to_bytes(user_res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let user: User = serde_json::from_slice(&user_bytes).unwrap();

        // 2. Add task for user
        let task_req = Request::builder()
            .method("POST")
            .uri(format!("/api/users/{}/tasks", user.id))
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&CreateTaskPayload {
                    title: "Eve's first task".to_string(),
                    description: Some("Work on Rust backend".to_string()),
                })
                .unwrap(),
            ))
            .unwrap();
        let task_res = app.clone().oneshot(task_req).await.unwrap();
        assert_eq!(task_res.status(), StatusCode::CREATED);

        // 3. List tasks
        let list_tasks_req = Request::builder()
            .uri(format!("/api/users/{}/tasks", user.id))
            .body(axum::body::Body::empty())
            .unwrap();
        let list_tasks_res = app.clone().oneshot(list_tasks_req).await.unwrap();
        assert_eq!(list_tasks_res.status(), StatusCode::OK);
        let tasks_bytes = axum::body::to_bytes(list_tasks_res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let tasks: Vec<Task> = serde_json::from_slice(&tasks_bytes).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Eve's first task");

        // 4. Delete user and check cascade
        let del_user_req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/users/{}", user.id))
            .body(axum::body::Body::empty())
            .unwrap();
        let del_user_res = app.clone().oneshot(del_user_req).await.unwrap();
        assert_eq!(del_user_res.status(), StatusCode::NO_CONTENT);

        // 5. Querying tasks for deleted user returns 404
        let check_tasks_req = Request::builder()
            .uri(format!("/api/users/{}/tasks", user.id))
            .body(axum::body::Body::empty())
            .unwrap();
        let check_tasks_res = app.oneshot(check_tasks_req).await.unwrap();
        assert_eq!(check_tasks_res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_bearer_auth_protection() {
        let app = test_app().await;

        // 1. Missing Authorization header -> 401
        let no_auth_req = Request::builder()
            .uri("/api/admin/stats")
            .body(axum::body::Body::empty())
            .unwrap();
        let no_auth_res = app.clone().oneshot(no_auth_req).await.unwrap();
        assert_eq!(no_auth_res.status(), StatusCode::UNAUTHORIZED);

        // 2. Invalid Bearer token -> 401
        let bad_auth_req = Request::builder()
            .uri("/api/admin/stats")
            .header("Authorization", "Bearer wrong-token")
            .body(axum::body::Body::empty())
            .unwrap();
        let bad_auth_res = app.clone().oneshot(bad_auth_req).await.unwrap();
        assert_eq!(bad_auth_res.status(), StatusCode::UNAUTHORIZED);

        // 3. Valid Bearer token -> 200 OK
        let valid_auth_req = Request::builder()
            .uri("/api/admin/stats")
            .header("Authorization", "Bearer secret-token-123")
            .body(axum::body::Body::empty())
            .unwrap();
        let valid_auth_res = app.oneshot(valid_auth_req).await.unwrap();
        assert_eq!(valid_auth_res.status(), StatusCode::OK);
    }
}
