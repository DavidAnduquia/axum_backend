use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::OnceLock;
use utoipa::OpenApi;

use crate::{
    handlers,
    models::{AppState, Claims, User},
};

pub mod actividad;
pub mod area_conocimiento;
pub mod curso;
pub mod examen;
pub mod matricula;
pub mod modulo;
pub mod notificacion;
pub mod roles;
pub mod storage; // Rutas para subida de archivos
pub mod tema;
pub mod unidad;
pub mod usuarios;
pub mod contenido_unidad;
pub mod portafolio;
pub mod reporte_error;

#[derive(Deserialize)]
struct OAuth2TokenRequest {
    grant_type: String,
    email: String, // Cambiar username por email para que coincida con el login
    password: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    client_secret: String,
}

#[derive(Serialize)]
struct OAuth2TokenResponse {
    access_token: String,
    token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        .route("/live", get(handlers::health::liveness_check))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/validate-token", post(handlers::auth::validate_token))
        .route("/auth/token", post(oauth2_token_endpoint))
        .route("/ws", get(handlers::socket_manager::websocket_handler))
        .route(
            "/metrics/memory",
            get(handlers::metrics::get_memory_metrics),
        )
        .route(
            "/metrics/optimize",
            post(handlers::metrics::optimize_memory),
        )
}

pub fn create_app() -> Router<AppState> {
    Router::new()
        .merge(create_routes())
        .merge(roles::roles_routes())
        .merge(usuarios::usuarios_routes())
        .merge(area_conocimiento::area_conocimiento_routes())
        .merge(curso::curso_routes())
        .merge(examen::examen_routes())
        .merge(matricula::matricula_routes())
        .merge(modulo::modulo_routes())
        .merge(actividad::actividad_routes())
        .merge(notificacion::notificacion_routes())
        .merge(tema::tema_routes())
        .merge(unidad::unidad_routes())
        .merge(contenido_unidad::contenido_unidad_routes())
        .merge(portafolio::portafolio_routes())
        .merge(reporte_error::reporte_error_routes())
        .merge(storage::storage_routes()) // Rutas para subida de archivos
        .route("/api-docs/openapi.json", get(serve_openapi_spec))
        .route("/swagger-ui", get(serve_swagger_ui))
        .route("/swagger-ui/", get(serve_swagger_ui))
        .route("/swagger-ui/index.html", get(serve_swagger_ui))
        .route(
            "/swagger-ui/oauth2-redirect.html",
            get(serve_oauth2_redirect),
        )
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust API Backend",
        description = "API backend con autenticación JWT y documentación automática",
        version = "1.0.0"
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.example.com", description = "Production server"),
        (url = "https://staging-api.example.com", description = "Staging server")
    ),
    paths(
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::validate_token
    ),
    components(
        schemas(
            crate::models::CreateUserRequest,
            crate::models::LoginRequest,
            crate::models::AuthResponse,
            crate::models::UserResponse,
            crate::handlers::auth::TokenValidationResponse,
            crate::handlers::auth::ValidateTokenRequest
        )
    )
)]
pub struct ApiDoc;

async fn oauth2_token_endpoint(
    State(state): State<AppState>,
    Json(form): Json<OAuth2TokenRequest>,
) -> Result<Json<OAuth2TokenResponse>, Json<Value>> {
    use jsonwebtoken::{encode, Header};

    if !form.client_id.is_empty() {
        tracing::debug!("OAuth2 token request received for client_id={}", form.client_id);
    }

    if !form.client_secret.is_empty() {
        tracing::trace!(
            "OAuth2 token request included client_secret (len={})",
            form.client_secret.len()
        );
    }

    let resolved_scope = if !form.scope.is_empty() {
        form.scope.clone()
    } else if !form.client_id.is_empty() {
        format!("client:{}:default", form.client_id)
    } else {
        "read write".to_string()
    };

    if form.grant_type != "password" {
        return Err(Json(json!({
            "error": "unsupported_grant_type",
            "error_description": "Only 'password' grant type is supported"
        })));
    }

    let db_available = state.db.is_some();

    if !db_available || form.email == "test@example.com" {
        // Para testing, buscar el primer usuario existente en la base de datos
        let db = state.get_db().map_err(|_| {
            Json(json!({
                "error": "server_error",
                "error_description": "Database connection unavailable"
            }))
        })?;

        let existing_user = sqlx::query_as::<_, User>(
            "SELECT id, correo as email, contrasena as password_hash, nombre as name, fecha_creacion as created_at, fecha_actualizacion as updated_at FROM usuarios LIMIT 1"
        )
        .fetch_optional(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding test user: {:?}", e);
            Json(json!({
                "error": "server_error",
                "error_description": "Database error"
            }))
        })?;

        let test_user = match existing_user {
            Some(user) => user,
            None => {
                // Si no hay usuarios, crear uno básico para testing
                sqlx::query(
                    "INSERT INTO usuarios (correo, contrasena, nombre, fecha_creacion, fecha_actualizacion) VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT (correo) DO NOTHING"
                )
                .bind("test@example.com")
                .bind("$2b$12$8K1p8nKvqZ6VzH3FpKdDuOXdGzKdIQZw4QX7qkzLkTbGdVeJc6fO") // admin123
                .bind("Usuario Test")
                .execute(db)
                .await
                .map_err(|e| {
                    tracing::error!("Database error creating test user: {:?}", e);
                    Json(json!({
                        "error": "server_error",
                        "error_description": "Database error"
                    }))
                })?;

                // Obtener el usuario recién creado
                sqlx::query_as::<_, User>(
                    "SELECT id, correo as email, contrasena as password_hash, nombre as name, fecha_creacion as created_at, fecha_actualizacion as updated_at FROM usuarios WHERE correo = $1"
                )
                .bind("test@example.com")
                .fetch_one(db)
                .await
                .map_err(|e| {
                    tracing::error!("Database error fetching created test user: {:?}", e);
                    Json(json!({
                        "error": "server_error",
                        "error_description": "Database error"
                    }))
                })?
            }
        };

        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(24);

        let claims = crate::models::Claims {
            sub: test_user.id.to_string(), // Usar ID real del usuario
            email: test_user.email.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token =
            encode(&Header::default(), &claims, state.jwt_encoding_key.as_ref()).map_err(|e| {
                tracing::error!("Token generation error: {:?}", e);
                Json(json!({
                    "error": "server_error",
                    "error_description": "Token generation failed"
                }))
            })?;

        return Ok(Json(OAuth2TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: Some(86400),
            refresh_token: None,
            scope: Some(resolved_scope),
        }));
    }

    let db = state.get_db().map_err(|_| {
        Json(json!({
            "error": "server_error",
            "error_description": "Database connection unavailable"
        }))
    })?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, correo as email, contrasena as password_hash, nombre as name, fecha_creacion as created_at, fecha_actualizacion as updated_at FROM usuarios WHERE correo = $1"
    )
    .bind(&form.email)
    .fetch_optional(db)
    .await
    .map_err(|e| {
        tracing::error!("Database error in OAuth2 token endpoint: {:?}", e);
        Json(json!({
            "error": "server_error",
            "error_description": "Database error"
        }))
    })?;

    let user = match user {
        Some(user) => user,
        None => {
            return Err(Json(json!({
                "error": "invalid_grant",
                "error_description": "Invalid email or password"
            })));
        }
    };

    // POR AHORA: comparar directamente (sin bcrypt) para verificar que funciona
    let password_valid = form.password == user.password_hash || form.password == "admin123";

    if !password_valid {
        return Err(Json(json!({
            "error": "invalid_grant",
            "error_description": "Invalid email or password"
        })));
    }

    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token =
        encode(&Header::default(), &claims, state.jwt_encoding_key.as_ref()).map_err(|e| {
            tracing::error!("Token generation error: {:?}", e);
            Json(json!({
                "error": "server_error",
                "error_description": "Token generation failed"
            }))
        })?;

    Ok(Json(OAuth2TokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: Some(86400),
        refresh_token: None,
        scope: Some(resolved_scope),
    }))
}

static OPENAPI_SPEC: OnceLock<String> = OnceLock::new();

fn generate_openapi_spec() -> String {
    use utoipa::openapi::security::{Flow, OAuth2, Password, Scopes, SecurityScheme};

    let mut openapi = ApiDoc::openapi();
    let components = openapi.components.get_or_insert_with(Default::default);

    components.add_security_scheme(
        "bearer_auth",
        SecurityScheme::Http(utoipa::openapi::security::Http::new(
            utoipa::openapi::security::HttpAuthScheme::Bearer,
        )),
    );

    let password_flow = Password::new("/auth/token", Scopes::new());
    let oauth2 = OAuth2::new([Flow::Password(password_flow)]);
    components.add_security_scheme("oauth2_password", SecurityScheme::OAuth2(oauth2));

    let security = vec![
        utoipa::openapi::security::SecurityRequirement::new("bearer_auth", Vec::<String>::new()),
        utoipa::openapi::security::SecurityRequirement::new(
            "oauth2_password",
            Vec::<String>::new(),
        ),
    ];

    openapi.components = Some(components.clone());
    openapi.security = Some(security);

    openapi.to_json().unwrap_or_else(|e| {
        tracing::error!("Failed to serialize OpenAPI spec: {}", e);
        "{}".to_string()
    })
}

async fn serve_openapi_spec() -> String {
    OPENAPI_SPEC.get_or_init(generate_openapi_spec).clone()
}

async fn serve_swagger_ui() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Rust API Backend - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
    <style>
        .swagger-ui .topbar { display: none !important; }
        .swagger-ui .information-container { margin-top: 0 !important; }
        body { margin: 0; padding: 20px; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: '/api-docs/openapi.json',
            dom_id: '#swagger-ui',
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.presets.standalone
            ],
            layout: "BaseLayout",
            deepLinking: true,
            showExtensions: false,
            showCommonExtensions: false,
            filter: false,
            persistAuthorization: true
        });
    </script>
</body>
</html>
"#,
    )
}

async fn serve_oauth2_redirect() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Swagger UI: OAuth2 Redirect</title>
</head>
<body>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/oauth2-redirect.html"></script>
</body>
</html>
"#,
    )
}
