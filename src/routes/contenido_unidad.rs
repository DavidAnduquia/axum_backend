use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers::contenido_unidad, models::AppState};

pub fn contenido_unidad_routes() -> Router<AppState> {
    Router::new()
        // Contenidos por unidad
        .route(
            "/api/unidades/{unidad_id}/contenidos",
            get(contenido_unidad::listar_contenidos_por_unidad),
        )
        // CRUD de contenido individual
        .route("/api/contenidos", post(contenido_unidad::crear_contenido))
        .route(
            "/api/contenidos/{id}",
            get(contenido_unidad::obtener_contenido)
                .put(contenido_unidad::actualizar_contenido)
                .delete(contenido_unidad::eliminar_contenido),
        )
}
