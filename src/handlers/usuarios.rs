use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    middleware::auth::AuthUser,
    models::{usuario as usuario_models, AppState, Claims},
    services::socket_service::get_socket_service,
    services::usuario_service::UsuarioService,
    utils::errors::AppError,
};

// POST /api/usuarios/login
#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub identificador: String, // correo o documento_nit
    pub contrasena: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    #[serde(flatten)]
    pub usuario: usuario_models::Model,
}

pub async fn login_usuario(
    State(state): State<AppState>,
    State(service): State<Arc<UsuarioService>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    let usuario = service
        .login_usuario(&payload.identificador, &payload.contrasena)
        .await?;

    // Generar token JWT
    let token = generate_token(&usuario, state.jwt_encoding_key.as_ref())?;

    Ok(Json(LoginResponse { token, usuario }))
}

fn generate_token(
    usuario: &usuario_models::Model,
    encoding_key: &jsonwebtoken::EncodingKey,
) -> Result<String, AppError> {
    use chrono::Utc;
    use jsonwebtoken::{encode, Header};

    let now = Utc::now();
    let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: usuario.id.to_string(),
        email: usuario.correo.clone(),
        exp,
        iat,
    };

    Ok(encode(&Header::default(), &claims, encoding_key)?)
}

// POST /api/usuarios/logout/:id
/// Cierra la sesión del usuario y desconecta todas sus conexiones WebSocket
/// Notifica a los demás usuarios conectados sobre el cambio
pub async fn logout_usuario(
    State(service): State<Arc<UsuarioService>>,
    Path(id): Path<i32>,
) -> Result<Json<usuario_models::Model>, AppError> {
    // Obtener información del usuario antes de cerrar sesión
    let usuario = service.logout_usuario(id).await?;
    
    // Obtener el servicio de sockets
    let socket_service = get_socket_service();
    
    // Cerrar todas las conexiones WebSocket del usuario
    socket_service.disconnect_all_user_sockets(id as i64).await;
    
    // Notificar a todos los demás usuarios que este usuario se desconectó
    let disconnect_notification = serde_json::json!({
        "event": "users_updated",
        "data": {
            "action": "user_disconnected",
            "user_id": id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    socket_service.emit_notification_broadcast(disconnect_notification).await;
    
    tracing::info!(
        "👤 Usuario {} ({}) cerró sesión manualmente",
        id,
        usuario.nombre
    );
    
    Ok(Json(usuario))
}

// GET /api/usuarios
pub async fn listar_usuarios(
    _auth_user: AuthUser, // Validar JWT automáticamente
    State(service): State<Arc<UsuarioService>>,
) -> Result<Json<Vec<usuario_models::UsuarioConRol>>, AppError> {
    let usuarios = service.obtener_usuarios().await?;
    Ok(Json(usuarios))
}

// POST /api/usuarios
pub async fn crear_usuario(
    _auth_user: AuthUser, // Validar JWT automáticamente
    State(service): State<Arc<UsuarioService>>,
    Json(payload): Json<usuario_models::NewUsuario>,
) -> Result<Json<usuario_models::Model>, AppError> {
    let usuario = service.crear_usuario(payload).await?;
    Ok(Json(usuario))
}

// PUT /api/usuarios/:id
pub async fn actualizar_usuario(
    _auth_user: AuthUser, // Validar JWT automáticamente
    Path(id): Path<i32>,
    State(service): State<Arc<UsuarioService>>,
    Json(payload): Json<usuario_models::UpdateUsuario>,
) -> Result<Json<usuario_models::Model>, AppError> {
    let usuario = service.editar_usuario(id, payload).await?;
    Ok(Json(usuario))
}

// GET /api/usuarios/:id
pub async fn obtener_usuario_por_id(
    _auth_user: AuthUser, // Validar JWT automáticamente
    Path(id): Path<i32>,
    State(service): State<Arc<UsuarioService>>,
) -> Result<Json<Option<usuario_models::Model>>, AppError> {
    let usuario = service.obtener_usuario_por_id(id).await?;
    Ok(Json(usuario))
}

// GET /api/usuarios/conectados
/// Obtiene la lista de usuarios actualmente conectados vía WebSocket
/// junto con su información completa de la base de datos
pub async fn listar_usuarios_conectados(
    _auth_user: AuthUser, // Solo usuarios autenticados pueden ver usuarios conectados
    State(service): State<Arc<UsuarioService>>,
) -> Result<Json<Vec<usuario_models::UsuarioConectadoInfo>>, AppError> {
    // Obtener conexiones activas del SocketService
    let socket_service = get_socket_service();
    let connection_info = socket_service.get_connection_info().await;

    // Extraer los user_ids de las rooms (formato "user_{id}")
    let user_ids: Vec<i32> = connection_info
        .rooms
        .iter()
        .filter_map(|room| {
            room.strip_prefix("user_")
                .and_then(|id_str| id_str.parse::<i32>().ok())
        })
        .collect();

    // Obtener información completa de los usuarios desde la base de datos
    let usuarios_conectados = service.obtener_usuarios_conectados(user_ids).await?;

    tracing::info!(
        "📊 Listados {} usuarios conectados (total: {})",
        usuarios_conectados.len(),
        connection_info.connected_users
    );

    Ok(Json(usuarios_conectados))
}

// POST /api/usuarios/{id}/expirar-sesion
/// Expira la sesión de un usuario específico (solo para administradores)
/// Esto invalida su token JWT actual forzando un nuevo login
pub async fn expirar_sesion_usuario(
    _auth_user: AuthUser, // Validar que tenga permisos de admin en middleware
    Path(id): Path<i32>,
    State(service): State<Arc<UsuarioService>>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::services::socket_service::get_socket_service;
    use serde_json::json;

    // Actualizar fecha de última conexión para invalidar tokens
    let usuario = service.expirar_sesion_usuario(id).await?;

    // Enviar notificación al usuario vía WebSocket
    let socket_service = get_socket_service();
    let notification = json!({
        "event": "session_expired",
        "data": {
            "message": "Se le ha expirado la sesión manualmente",
            "user_id": id,
            "redirect_after": 3, // segundos
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    socket_service.emit_notification_to_user(id as i64, notification).await;

    // Notificar a todos los usuarios conectados que la lista debe actualizarse
    let refresh_notification = json!({
        "event": "users_updated",
        "data": {
            "action": "user_disconnected",
            "user_id": id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    socket_service.emit_notification_broadcast(refresh_notification).await;

    // Marcar la sesión como expirada y cerrar conexiones WebSocket
    // Esto garantiza que el usuario no aparezca en la lista de conectados
    socket_service.expire_user_session(id as i64).await;

    tracing::info!("🚫 Sesión expirada para usuario {} ({}) por administrador", id, usuario.nombre);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Sesión de {} expirada exitosamente", usuario.nombre),
        "user_id": id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
