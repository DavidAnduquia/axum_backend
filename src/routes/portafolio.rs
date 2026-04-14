use axum::{routing::get, Router};

use crate::{handlers::portafolio, models::AppState};

pub fn portafolio_routes() -> Router<AppState> {
    Router::new().route(
        "/api/cursos/{curso_id}/portafolio/portada",
        get(portafolio::obtener_portada_curso).put(portafolio::guardar_portada_curso),
    )
}
