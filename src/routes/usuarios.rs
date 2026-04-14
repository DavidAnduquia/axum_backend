use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers::usuarios, models::AppState};

pub fn usuarios_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/usuarios",
            get(usuarios::listar_usuarios).post(usuarios::crear_usuario),
        )
        .route(
            "/api/usuarios/{id}",
            get(usuarios::obtener_usuario_por_id).put(usuarios::actualizar_usuario),
        )
        .route("/api/usuarios/conectados", get(usuarios::listar_usuarios_conectados))
        .route("/api/usuarios/{id}/expirar-sesion", post(usuarios::expirar_sesion_usuario))
        .route("/api/usuario/logout/{id}", post(usuarios::logout_usuario))
        .route("/api/usuario/login", post(usuarios::login_usuario))
}
