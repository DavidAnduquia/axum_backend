use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{handlers::notificacion, models::AppState};

pub fn notificacion_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/notificaciones/usuario/{usuario_id}",
            get(notificacion::obtener_notificaciones_usuario),
        )
        .route(
            "/api/notificaciones/{id}/leida",
            put(notificacion::marcar_notificacion_leida),
        )
        .route(
            "/api/notificaciones",
            post(notificacion::crear_notificacion),
        )
        .route(
            "/api/notificaciones/usuario/{usuario_id}/marcar-todas-leidas",
            put(notificacion::marcar_todas_leidas),
        )
}
