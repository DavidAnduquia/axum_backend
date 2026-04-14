use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers::reporte_error, models::AppState};

pub fn reporte_error_routes() -> Router<AppState> {
    Router::new()
        // CRUD Básico
        .route("/api/reportes", post(reporte_error::crear_reporte))
        .route("/api/reportes", get(reporte_error::listar_reportes))
        .route("/api/reportes/{id}", get(reporte_error::obtener_reporte))
        .route("/api/reportes/{id}", put(reporte_error::actualizar_reporte))
        .route("/api/reportes/{id}", delete(reporte_error::eliminar_reporte))
        // Lógica de negocio - Estados y Asignación
        .route(
            "/api/reportes/{id}/cambiar-estado",
            put(reporte_error::cambiar_estado_reporte),
        )
        .route(
            "/api/reportes/{id}/asignar-responsable",
            put(reporte_error::asignar_responsable),
        )
        // Seguimiento
        .route(
            "/api/reportes/{reporte_id}/seguimientos",
            post(reporte_error::crear_seguimiento),
        )
        .route(
            "/api/reportes/{reporte_id}/seguimientos",
            get(reporte_error::obtener_seguimientos),
        )
        // Estadísticas
        .route(
            "/api/reportes/estadisticas",
            get(reporte_error::obtener_estadisticas),
        )
        // Consultas específicas por usuario
        .route("/api/mis-reportes", get(reporte_error::mis_reportes))
        .route(
            "/api/mis-reportes-asignados",
            get(reporte_error::reportes_asignados),
        )
}
