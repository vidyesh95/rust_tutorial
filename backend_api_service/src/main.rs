//! # Phase 4: Production-Grade REST API with Axum, Serde, SQLx & BetterAuth Compatibility
//!
//! ## Key Backend Architecture:
//! - **BetterAuth Structure**: Standard `/api/auth/*` endpoints (`sign-up/email`, `sign-in/email`,
//!   `sign-out`, `get-session`, `me`) with HTTP-only session cookies (`better-auth.session_token`)
//!   and Bearer token fallback.
//! - **Password Security**: Salted SHA-256 cryptographic hashing (`salt$hash`) and verification.
//! - **Axum Extractors**: `BetterAuthUser` and `AuthUser` implementing `FromRequestParts` to inject
//!   authenticated users into route handlers cleanly.
//! - **SQLx SQLite**: Non-blocking asynchronous SQL execution, connection pooling (`SqlitePool`),
//!   foreign key enforcement, and atomic multi-table transactions (`pool.begin()`).
//! - **OpenAPI & Swagger UI (utoipa)**: Interactive documentation served directly at `/swagger-ui`
//!   and machine-readable specification served at `/api-docs/openapi.json`.
//! - **Structured Domain Errors**: `AppError` implements `IntoResponse`, mapping internal/validation/auth
//!   errors directly to HTTP status codes (400, 401, 404, 409, 500) and consistent JSON responses.

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{COOKIE, SET_COOKIE},
        request::Parts,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    FromRow, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::trace::TraceLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

// --- 1. Shared Application State ---

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

// --- 2. Cryptographic Helpers (Password Hashing & Session Tokens) ---

/// Hashes a plaintext password with a random 16-hex-digit salt using SHA-256.
/// Output format matches standard crypto formats: `<salt>$<sha256_hex>`
pub fn hash_password(password: &str) -> String {
    let salt: String = (0..16)
        .map(|_| format!("{:x}", rand::random::<u8>() % 16))
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let hash = hex::encode(hasher.finalize());
    format!("{salt}${hash}")
}

/// Verifies whether a plaintext password matches a stored `<salt>$<hash>` string.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parts: Vec<&str> = stored_hash.split('$').collect();
    if parts.len() != 2 {
        return false;
    }
    let salt = parts[0];
    let expected_hash = parts[1];

    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let computed_hash = hex::encode(hasher.finalize());
    computed_hash == expected_hash
}

/// Generates a cryptographically random 48-character hex session token.
pub fn generate_session_token() -> String {
    let bytes: [u8; 24] = rand::random();
    hex::encode(bytes)
}

/// Returns current UNIX timestamp in seconds.
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// --- 3. Domain Models & DTOs ---

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct User {
    /// Unique database identifier
    #[schema(example = 1)]
    pub id: i64,
    /// User's display name
    #[schema(example = "Alice Doe")]
    pub name: String,
    /// Primary email address
    #[schema(example = "alice@example.com")]
    pub email: String,
    /// Unix timestamp of account creation
    #[schema(example = 1716000000)]
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct Session {
    /// Unique session identifier
    #[schema(example = 1)]
    pub id: i64,
    /// Foreign key referencing users(id)
    #[schema(example = 1)]
    pub user_id: i64,
    /// Random opaque session token
    #[schema(example = "4f8a3c9b7e1d2f0a5b6c8d7e9f0a1b2c3d4e5f6a")]
    pub token: String,
    /// Unix timestamp when this session expires
    #[schema(example = 1716604800)]
    pub expires_at: i64,
    /// Unix timestamp when this session was created
    #[schema(example = 1716000000)]
    pub created_at: i64,
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
pub struct SignUpPayload {
    /// User's full name
    #[schema(example = "Alex Doe")]
    pub name: String,
    /// Valid email address
    #[schema(example = "alex@example.com")]
    pub email: String,
    /// Secret password (minimum 6 characters)
    #[schema(example = "supersecret123")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignInPayload {
    /// User's email address
    #[schema(example = "alex@example.com")]
    pub email: String,
    /// User's password
    #[schema(example = "supersecret123")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// The generated session token
    #[schema(example = "4f8a3c9b7e1d2f0a5b6c8d7e9f0a1b2c3d4e5f6a")]
    pub token: String,
    /// The authenticated user record
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionResponse {
    /// Session metadata
    pub session: Session,
    /// Associated user profile
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserPayload {
    /// Desired name
    #[schema(example = "bob")]
    pub username: String,
    /// User's contact email
    #[schema(example = "bob@example.com")]
    pub email: String,
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
    #[schema(example = "SQLite / SQLx (Foreign Keys Enabled)")]
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
            return AppError::Conflict("A user with this email already exists".to_string());
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

// --- 5. Authentication Extractors (BetterAuth & Bearer Token Patterns) ---

/// Helper function to extract a session token from either:
/// 1. Cookie: `better-auth.session_token=<token>`
/// 2. Header: `Authorization: Bearer <token>`
pub fn extract_token_from_parts(parts: &Parts) -> Option<String> {
    // 1. Try Cookie header first
    if let Some(cookie_header) = parts.headers.get(COOKIE).and_then(|v| v.to_str().ok()) {
        for item in cookie_header.split(';') {
            let item = item.trim();
            if let Some((name, val)) = item.split_once('=')
                && name == "better-auth.session_token"
            {
                return Some(val.to_string());
            }
        }
    }

    // 2. Fall back to Authorization: Bearer <token>
    if let Some(auth_header) = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        && let Some(token) = auth_header.strip_prefix("Bearer ")
    {
        return Some(token.trim().to_string());
    }

    None
}

/// Extractor that verifies the BetterAuth session from Cookies or Bearer headers against SQLx database.
#[derive(Debug, Clone)]
pub struct BetterAuthUser {
    pub user: User,
    pub session: Session,
}

impl FromRequestParts<Arc<AppState>> for BetterAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token_from_parts(parts).ok_or_else(|| {
            AppError::Unauthorized(
                "Missing authentication. Provide 'better-auth.session_token' cookie or Bearer token."
                    .to_string(),
            )
        })?;

        let now = current_timestamp();

        let session = sqlx::query_as::<_, Session>(
            "SELECT id, user_id, token, expires_at, created_at FROM sessions WHERE token = ? AND expires_at > ?",
        )
        .bind(&token)
        .bind(now)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".to_string()))?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, created_at FROM users WHERE id = ?",
        )
        .bind(session.user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User belonging to session no longer exists".to_string()))?;

        Ok(BetterAuthUser { user, session })
    }
}

/// Legacy / Static Bearer token extractor for administrative tooling.
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

// --- 6. Database Setup & Migrations ---

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
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            token TEXT NOT NULL UNIQUE,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
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

// --- 7. BetterAuth Endpoint Handlers ---

/// Sign up a new user with email, password, and name (BetterAuth compatible)
#[utoipa::path(
    post,
    path = "/api/auth/sign-up/email",
    tag = "BetterAuth",
    request_body = SignUpPayload,
    responses(
        (status = 201, description = "User registered and session created", body = AuthResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Email already registered", body = ErrorResponse),
        (status = 500, description = "Database failure", body = ErrorResponse)
    )
)]
pub async fn auth_sign_up(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignUpPayload>,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    let name = payload.name.trim();
    let email = payload.email.trim();
    let password = payload.password.trim();

    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }
    if password.len() < 6 {
        return Err(AppError::BadRequest(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    let hashed_pw = hash_password(password);
    let now = current_timestamp();

    // Begin atomic transaction: user + initial session
    let mut tx = state.pool.begin().await?;

    let user_id = sqlx::query(
        "INSERT INTO users (name, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(name)
    .bind(email)
    .bind(&hashed_pw)
    .bind(now)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    let token = generate_session_token();
    let expires_at = now + (7 * 24 * 3600); // 7 days expiration

    sqlx::query(
        "INSERT INTO sessions (user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(&token)
    .bind(expires_at)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let user = User {
        id: user_id,
        name: name.to_string(),
        email: email.to_string(),
        created_at: now,
    };

    // Format BetterAuth HTTP-Only session cookie
    let mut headers = HeaderMap::new();
    let cookie_val = format!(
        "better-auth.session_token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        7 * 24 * 3600
    );
    headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());

    Ok((
        StatusCode::CREATED,
        headers,
        Json(AuthResponse { token, user }),
    ))
}

#[derive(Debug, FromRow)]
struct UserAuthRow {
    id: i64,
    name: String,
    email: String,
    password_hash: String,
    created_at: i64,
}

/// Sign in an existing user with email and password (BetterAuth compatible)
#[utoipa::path(
    post,
    path = "/api/auth/sign-in/email",
    tag = "BetterAuth",
    request_body = SignInPayload,
    responses(
        (status = 200, description = "Sign in successful, returns user and session token", body = AuthResponse),
        (status = 401, description = "Invalid email or password", body = ErrorResponse),
        (status = 500, description = "Database failure", body = ErrorResponse)
    )
)]
pub async fn auth_sign_in(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignInPayload>,
) -> Result<(StatusCode, HeaderMap, Json<AuthResponse>), AppError> {
    let email = payload.email.trim();
    let password = payload.password.trim();

    let record = sqlx::query_as::<_, UserAuthRow>(
        "SELECT id, name, email, password_hash, created_at FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify_password(password, &record.password_hash) {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let token = generate_session_token();
    let now = current_timestamp();
    let expires_at = now + (7 * 24 * 3600);

    sqlx::query(
        "INSERT INTO sessions (user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(record.id)
    .bind(&token)
    .bind(expires_at)
    .bind(now)
    .execute(&state.pool)
    .await?;

    let user = User {
        id: record.id,
        name: record.name,
        email: record.email,
        created_at: record.created_at,
    };

    let mut headers = HeaderMap::new();
    let cookie_val = format!(
        "better-auth.session_token={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        7 * 24 * 3600
    );
    headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());

    Ok((StatusCode::OK, headers, Json(AuthResponse { token, user })))
}

/// Sign out and invalidate current session (BetterAuth compatible)
#[utoipa::path(
    post,
    path = "/api/auth/sign-out",
    tag = "BetterAuth",
    responses(
        (status = 200, description = "Session invalidated and cookie cleared"),
        (status = 401, description = "No active session", body = ErrorResponse)
    )
)]
pub async fn auth_sign_out(
    State(state): State<Arc<AppState>>,
    auth: BetterAuthUser,
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(auth.session.id)
        .execute(&state.pool)
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_static("better-auth.session_token=; Path=/; HttpOnly; Max-Age=0"),
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(serde_json::json!({ "success": true })),
    ))
}

/// Retrieve the active session and user profile (BetterAuth compatible)
#[utoipa::path(
    get,
    path = "/api/auth/get-session",
    tag = "BetterAuth",
    responses(
        (status = 200, description = "Active session information", body = SessionResponse),
        (status = 401, description = "Unauthorized or expired session", body = ErrorResponse)
    )
)]
pub async fn auth_get_session(
    auth: BetterAuthUser,
) -> Result<Json<SessionResponse>, AppError> {
    Ok(Json(SessionResponse {
        session: auth.session,
        user: auth.user,
    }))
}

/// Retrieve current authenticated user profile using the BetterAuthUser extractor
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "BetterAuth",
    responses(
        (status = 200, description = "Current authenticated user profile", body = User),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn auth_me(auth: BetterAuthUser) -> Result<Json<User>, AppError> {
    Ok(Json(auth.user))
}

// --- 8. Standard CRUD & Relational Route Handlers ---

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

/// Create a single user
#[utoipa::path(
    post,
    path = "/api/users",
    tag = "Users",
    request_body = CreateUserPayload,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Email conflict", body = ErrorResponse),
        (status = 500, description = "Internal database error", body = ErrorResponse)
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let name = payload.username.trim();
    let email = payload.email.trim();

    if name.is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    let default_hash = hash_password("default123");
    let now = current_timestamp();

    let inserted_id = sqlx::query(
        "INSERT INTO users (name, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(name)
    .bind(email)
    .bind(default_hash)
    .bind(now)
    .execute(&state.pool)
    .await?
    .last_insert_rowid();

    let user = User {
        id: inserted_id,
        name: name.to_string(),
        email: email.to_string(),
        created_at: now,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

/// Create a user along with multiple initial tasks atomically in a single SQL transaction.
#[utoipa::path(
    post,
    path = "/api/users/atomic",
    tag = "Transactions",
    request_body = CreateUserWithTasksPayload,
    responses(
        (status = 201, description = "User and tasks created atomically", body = UserWithTasks),
        (status = 400, description = "Validation failure (triggers automatic rollback)", body = ErrorResponse),
        (status = 409, description = "Email conflict", body = ErrorResponse),
        (status = 500, description = "Database error", body = ErrorResponse)
    )
)]
pub async fn create_user_with_tasks_atomic(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserWithTasksPayload>,
) -> Result<(StatusCode, Json<UserWithTasks>), AppError> {
    let name = payload.username.trim();
    let email = payload.email.trim();

    if name.is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    let default_hash = hash_password("default123");
    let now = current_timestamp();

    // Begin an atomic transaction:
    let mut tx = state.pool.begin().await?;

    // Step 1: Insert user inside the transaction
    let user_id = sqlx::query(
        "INSERT INTO users (name, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(name)
    .bind(email)
    .bind(default_hash)
    .bind(now)
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
        name: name.to_string(),
        email: email.to_string(),
        created_at: now,
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
        "SELECT id, name, email, created_at FROM users ORDER BY id ASC LIMIT ? OFFSET ?",
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
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at FROM users WHERE id = ?",
    )
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

// --- 9. OpenAPI Documentation Specification ---

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        auth_sign_up,
        auth_sign_in,
        auth_sign_out,
        auth_get_session,
        auth_me,
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
            Session,
            Task,
            SignUpPayload,
            SignInPayload,
            AuthResponse,
            SessionResponse,
            CreateUserPayload,
            CreateUserWithTasksPayload,
            UserWithTasks,
            CreateTaskPayload,
            HealthResponse,
            ErrorResponse,
            AdminStatsResponse
        )
    ),
    tags(
        (name = "BetterAuth", description = "TypeScript BetterAuth compatible authentication endpoints"),
        (name = "System", description = "System and health check endpoints"),
        (name = "Users", description = "User management and CRUD endpoints"),
        (name = "Tasks", description = "Task management and foreign key relationships"),
        (name = "Transactions", description = "Atomic multi-table SQL transaction operations"),
        (name = "Auth", description = "Administrative endpoints requiring HTTP Bearer authentication")
    ),
    info(
        title = "Rust Tutorial Backend API",
        version = "1.0.0",
        description = "Production-grade asynchronous REST API built with Axum, SQLx SQLite, BetterAuth compatibility, atomic transactions, and Utoipa OpenAPI documentation."
    )
)]
pub struct ApiDoc;

// --- 10. Application Router Builder ---

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health_check))
        // BetterAuth compatible endpoints
        .route("/api/auth/sign-up/email", post(auth_sign_up))
        .route("/api/auth/sign-in/email", post(auth_sign_in))
        .route("/api/auth/sign-out", post(auth_sign_out))
        .route("/api/auth/get-session", get(auth_get_session))
        .route("/api/auth/session", get(auth_get_session)) // Alias
        .route("/api/auth/me", get(auth_me))
        // Standard user & transactional endpoints
        .route("/api/users", post(create_user).get(list_users))
        .route("/api/users/atomic", post(create_user_with_tasks_atomic))
        .route("/api/users/{id}", get(get_user).delete(delete_user))
        .route(
            "/api/users/{id}/tasks",
            post(create_task).get(list_user_tasks),
        )
        // Admin endpoints
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
    println!("\nBetterAuth Endpoints:");
    println!("  POST   http://{addr}/api/auth/sign-up/email");
    println!("  POST   http://{addr}/api/auth/sign-in/email");
    println!("  POST   http://{addr}/api/auth/sign-out");
    println!("  GET    http://{addr}/api/auth/get-session");
    println!("  GET    http://{addr}/api/auth/me");
    println!("\nCRUD & Relational Endpoints:");
    println!("  GET    http://{addr}/health");
    println!("  POST   http://{addr}/api/users");
    println!("  POST   http://{addr}/api/users/atomic (Transaction: user + tasks)");
    println!("  GET    http://{addr}/api/users?limit=10&offset=0");
    println!("  GET    http://{addr}/api/users/:id");
    println!("  DELETE http://{addr}/api/users/:id");
    println!("  POST   http://{addr}/api/users/:id/tasks");
    println!("  GET    http://{addr}/api/users/:id/tasks");

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
    async fn test_betterauth_sign_up_and_cookie() {
        let app = test_app().await;

        let payload = SignUpPayload {
            name: "BetterAuth Tester".to_string(),
            email: "tester@betterauth.dev".to_string(),
            password: "secure_password_123".to_string(),
        };

        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/sign-up/email")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&payload).unwrap(),
            ))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        // Verify Set-Cookie header is properly formatted
        let cookie_header = res
            .headers()
            .get(SET_COOKIE)
            .expect("Expected Set-Cookie header")
            .to_str()
            .unwrap();
        assert!(cookie_header.contains("better-auth.session_token="));
        assert!(cookie_header.contains("HttpOnly"));
        assert!(cookie_header.contains("Path=/"));

        let body_bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let auth_res: AuthResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(auth_res.user.email, "tester@betterauth.dev");
        assert!(!auth_res.token.is_empty());
    }

    #[tokio::test]
    async fn test_betterauth_full_auth_lifecycle() {
        let app = test_app().await;

        // 1. Sign Up
        let signup_req = Request::builder()
            .method("POST")
            .uri("/api/auth/sign-up/email")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&SignUpPayload {
                    name: "Sam Lifecycle".to_string(),
                    email: "sam@example.com".to_string(),
                    password: "password123".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        let signup_res = app.clone().oneshot(signup_req).await.unwrap();
        assert_eq!(signup_res.status(), StatusCode::CREATED);

        // 2. Sign In with wrong password -> 401
        let bad_signin_req = Request::builder()
            .method("POST")
            .uri("/api/auth/sign-in/email")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&SignInPayload {
                    email: "sam@example.com".to_string(),
                    password: "wrong_password".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        let bad_signin_res = app.clone().oneshot(bad_signin_req).await.unwrap();
        assert_eq!(bad_signin_res.status(), StatusCode::UNAUTHORIZED);

        // 3. Sign In with correct password -> 200 OK + Cookie
        let good_signin_req = Request::builder()
            .method("POST")
            .uri("/api/auth/sign-in/email")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&SignInPayload {
                    email: "sam@example.com".to_string(),
                    password: "password123".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();
        let good_signin_res = app.clone().oneshot(good_signin_req).await.unwrap();
        assert_eq!(good_signin_res.status(), StatusCode::OK);

        let signin_bytes = axum::body::to_bytes(good_signin_res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let signin_body: AuthResponse = serde_json::from_slice(&signin_bytes).unwrap();
        let token = signin_body.token;

        // 4. Get Session using Cookie
        let get_session_req = Request::builder()
            .uri("/api/auth/get-session")
            .header("Cookie", format!("better-auth.session_token={token}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let get_session_res = app.clone().oneshot(get_session_req).await.unwrap();
        assert_eq!(get_session_res.status(), StatusCode::OK);

        let session_bytes = axum::body::to_bytes(get_session_res.into_body(), 1024 * 16)
            .await
            .unwrap();
        let session_res: SessionResponse = serde_json::from_slice(&session_bytes).unwrap();
        assert_eq!(session_res.user.email, "sam@example.com");
        assert_eq!(session_res.session.token, token);

        // 5. Access /api/auth/me using Bearer header
        let me_req = Request::builder()
            .uri("/api/auth/me")
            .header("Authorization", format!("Bearer {token}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let me_res = app.clone().oneshot(me_req).await.unwrap();
        assert_eq!(me_res.status(), StatusCode::OK);

        // 6. Sign Out
        let signout_req = Request::builder()
            .method("POST")
            .uri("/api/auth/sign-out")
            .header("Cookie", format!("better-auth.session_token={token}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let signout_res = app.clone().oneshot(signout_req).await.unwrap();
        assert_eq!(signout_res.status(), StatusCode::OK);

        // 7. Verify session is now invalidated (401)
        let get_after_signout = Request::builder()
            .uri("/api/auth/get-session")
            .header("Cookie", format!("better-auth.session_token={token}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let after_signout_res = app.oneshot(get_after_signout).await.unwrap();
        assert_eq!(after_signout_res.status(), StatusCode::UNAUTHORIZED);
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

        // 2. Duplicate Email Conflict (409)
        let dup_req = Request::builder()
            .method("POST")
            .uri("/api/users")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&CreateUserPayload {
                    username: "alice2".to_string(),
                    email: "alice@example.com".to_string(),
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
        assert_eq!(user_with_tasks.user.name, "charlie");
        assert_eq!(user_with_tasks.tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_atomic_transaction_rollback_on_invalid_task() {
        let app = test_app().await;

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
            users.iter().all(|u| u.name != "dave_rollback"),
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
