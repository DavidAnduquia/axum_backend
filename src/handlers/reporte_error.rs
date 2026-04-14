use axum::{
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    middleware::auth::AuthUser,
    models::{
        reporte_error::{
            ActualizarReporteError, AsignarResponsableRequest, CambiarEstadoReporte,
            EstadoReporte, NuevoReporteError, NuevoSeguimientoReporte,
            PrioridadReporte, TipoReporte, TipoSeguimiento,
        },
        AppState,
    },
    services::reporte_error_service::ReporteErrorService,
    utils::errors::AppError,
};

// ==================== DTOs PARA REQUESTS ====================

#[derive(Debug, Deserialize)]
pub struct ListarReportesQuery {
    pub estado: Option<EstadoReporte>,
    pub tipo: Option<TipoReporte>,
    pub prioridad: Option<PrioridadReporte>,
    pub usuario_id: Option<i32>,
    pub responsable_id: Option<i32>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ListaReportesResponse {
    pub reportes: Vec<Value>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct SeguimientosResponse {
    pub seguimientos: Vec<Value>,
}

// ==================== SERIALIZADORES ====================

fn serialize_reporte<T: Serialize>(r: &T) -> Value {
    json!(r)
}

fn serialize_seguimiento<T: Serialize>(s: &T) -> Value {
    json!(s)
}

// ==================== HANDLERS - CRUD BÁSICO ====================

/// Crear un nuevo reporte de error/sugerencia
pub async fn crear_reporte(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(mut payload): Json<NuevoReporteError>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = ReporteErrorService::from_ref(&state);

    // Establecer el usuario_id del usuario autenticado
    payload.usuario_id = Some(auth_user.user_id);

    let reporte = service.crear_reporte(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": serialize_reporte(&reporte),
            "message": "Reporte creado exitosamente"
        })),
    ))
}

/// Obtener un reporte por ID
pub async fn obtener_reporte(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    let reporte = service.obtener_por_id(id).await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_reporte(&reporte)
    })))
}

/// Listar reportes con filtros
pub async fn listar_reportes(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListarReportesQuery>,
) -> Result<Json<ListaReportesResponse>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    let (reportes, total) = service
        .listar_reportes(
            params.estado,
            params.tipo,
            params.prioridad,
            params.usuario_id,
            params.responsable_id,
            params.limit,
            params.offset,
        )
        .await?;

    let reportes_json: Vec<Value> = reportes.iter().map(serialize_reporte).collect();

    Ok(Json(ListaReportesResponse {
        reportes: reportes_json,
        total,
    }))
}

/// Actualizar un reporte
pub async fn actualizar_reporte(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<ActualizarReporteError>,
) -> Result<Json<Value>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    let usuario_accion = Some(auth_user.user_id);

    let reporte = service
        .actualizar_reporte(id, payload, usuario_accion)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_reporte(&reporte),
        "message": "Reporte actualizado exitosamente"
    })))
}

/// Eliminar un reporte
pub async fn eliminar_reporte(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    service.eliminar_reporte(id).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Reporte eliminado exitosamente"
    })))
}

// ==================== HANDLERS - LÓGICA DE NEGOCIO ====================

/// Cambiar estado de un reporte
pub async fn cambiar_estado_reporte(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(mut payload): Json<CambiarEstadoReporte>,
) -> Result<Json<Value>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    payload.usuario_id = Some(auth_user.user_id);

    let reporte = service.cambiar_estado(id, payload).await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_reporte(&reporte),
        "message": "Estado del reporte actualizado"
    })))
}

/// Asignar responsable a un reporte
pub async fn asignar_responsable(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<AsignarResponsableRequest>,
) -> Result<Json<Value>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    let usuario_accion = Some(auth_user.user_id);

    let reporte = service
        .asignar_responsable(id, payload, usuario_accion)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_reporte(&reporte),
        "message": "Responsable asignado exitosamente"
    })))
}

// ==================== HANDLERS - SEGUIMIENTO ====================

/// Agregar comentario/seguimiento a un reporte
pub async fn crear_seguimiento(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reporte_id): Path<i32>,
    Json(mut payload): Json<NuevoSeguimientoReporte>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = ReporteErrorService::from_ref(&state);

    // Asignar el reporte_id de la URL y el usuario_id del autenticado
    payload.reporte_id = reporte_id;
    payload.usuario_id = Some(auth_user.user_id);

    // Si no se especifica tipo, usar Comentario por defecto
    if matches!(payload.tipo, TipoSeguimiento::Otro) && payload.estado_nuevo.is_none() {
        // Mantener el tipo que venga o usar Comentario
    }

    let seguimiento = service.crear_seguimiento(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": serialize_seguimiento(&seguimiento),
            "message": "Seguimiento agregado exitosamente"
        })),
    ))
}

/// Obtener seguimientos de un reporte
pub async fn obtener_seguimientos(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reporte_id): Path<i32>,
    Query(params): Query<ListarReportesQuery>,
) -> Result<Json<SeguimientosResponse>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    let seguimientos = service
        .obtener_seguimientos(reporte_id, params.limit)
        .await?;

    let seguimientos_json: Vec<Value> = seguimientos.iter().map(serialize_seguimiento).collect();

    Ok(Json(SeguimientosResponse {
        seguimientos: seguimientos_json,
    }))
}

// ==================== HANDLERS - ESTADÍSTICAS ====================

/// Obtener estadísticas de reportes
pub async fn obtener_estadisticas(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    let stats = service.obtener_estadisticas().await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "total": stats.total,
            "por_estado": {
                "recibidos": stats.recibidos,
                "en_revision": stats.en_revision,
                "en_desarrollo": stats.en_desarrollo,
                "resueltos": stats.resueltos,
                "rechazados": stats.rechazados
            },
            "criticos_pendientes": stats.criticos_pendientes
        }
    })))
}

// ==================== HANDLERS - MIS REPORTES ====================

/// Obtener reportes del usuario actual
pub async fn mis_reportes(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListarReportesQuery>,
) -> Result<Json<ListaReportesResponse>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    // Extraer usuario_id del token JWT
    // Asumiendo que AuthUser tiene el user_id
    let usuario_id = extract_user_id(&auth_user);

    let (reportes, total) = service
        .listar_reportes(
            params.estado,
            params.tipo,
            params.prioridad,
            usuario_id,
            params.responsable_id,
            params.limit,
            params.offset,
        )
        .await?;

    let reportes_json: Vec<Value> = reportes.iter().map(serialize_reporte).collect();

    Ok(Json(ListaReportesResponse {
        reportes: reportes_json,
        total,
    }))
}

/// Reportes asignados al usuario (para responsables)
pub async fn reportes_asignados(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListarReportesQuery>,
) -> Result<Json<ListaReportesResponse>, AppError> {
    let service = ReporteErrorService::from_ref(&state);

    // Extraer usuario_id del token JWT
    let responsable_id = extract_user_id(&auth_user);

    let (reportes, total) = service
        .listar_reportes(
            params.estado,
            params.tipo,
            params.prioridad,
            params.usuario_id,
            responsable_id,
            params.limit,
            params.offset,
        )
        .await?;

    let reportes_json: Vec<Value> = reportes.iter().map(serialize_reporte).collect();

    Ok(Json(ListaReportesResponse {
        reportes: reportes_json,
        total,
    }))
}

// Helper para extraer user_id del AuthUser
fn extract_user_id(auth_user: &AuthUser) -> Option<i32> {
    Some(auth_user.user_id)
}
