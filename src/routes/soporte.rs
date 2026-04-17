use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers::soporte, models::AppState};

pub fn soporte_routes() -> Router<AppState> {
    Router::new()
        // CRUD Básico
        .route("/api/soporte", post(soporte::crear_soporte))
        .route("/api/soporte", get(soporte::listar_soportes))
        .route("/api/soporte/{id}", get(soporte::obtener_soporte))
        .route("/api/soporte/{id}", put(soporte::actualizar_soporte))
        .route("/api/soporte/{id}", delete(soporte::eliminar_soporte))
        // Lógica de negocio - Estados y Asignación
        .route(
            "/api/soporte/{id}/cambiar-estado",
            put(soporte::cambiar_estado_soporte),
        )
        .route(
            "/api/soporte/{id}/asignar-responsable",
            put(soporte::asignar_responsable),
        )
        // Seguimiento
        .route(
            "/api/soporte/{soporte_id}/seguimientos",
            post(soporte::crear_seguimiento),
        )
        .route(
            "/api/soporte/{soporte_id}/seguimientos",
            get(soporte::obtener_seguimientos),
        )
        // Estadísticas
        .route(
            "/api/soporte/estadisticas",
            get(soporte::obtener_estadisticas),
        )
        // Consultas específicas por usuario
        .route("/api/mis-soportes", get(soporte::mis_soportes))
        .route(
            "/api/mis-soportes-asignados",
            get(soporte::soportes_asignados),
        )
}
