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
        soporte::{
            ActualizarSoporteError, AsignarResponsableRequest, CambiarEstadoSoporte,
            EstadoSoporte, NuevoSoporteError, NuevoSeguimientoSoporte,
            PrioridadSoporte, TipoSoporte, TipoSeguimientoSoporte,
        },
        AppState,
    },
    services::soporte_service::SoporteService,
    utils::errors::AppError,
};

// ==================== DTOs PARA REQUESTS ====================

#[derive(Debug, Deserialize)]
pub struct ListarSoportesQuery {
    pub estado: Option<EstadoSoporte>,
    pub tipo: Option<TipoSoporte>,
    pub prioridad: Option<PrioridadSoporte>,
    pub usuario_id: Option<i32>,
    pub responsable_id: Option<i32>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ListaSoportesResponse {
    pub soportes: Vec<Value>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct SeguimientosResponse {
    pub seguimientos: Vec<Value>,
}

// ==================== SERIALIZADORES ====================

fn serialize_soporte<T: Serialize>(r: &T) -> Value {
    json!(r)
}

fn serialize_seguimiento<T: Serialize>(s: &T) -> Value {
    json!(s)
}

// ==================== HANDLERS - CRUD BÁSICO ====================

/// Crear un nuevo reporte de error/sugerencia
pub async fn crear_soporte(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(mut payload): Json<NuevoSoporteError>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = SoporteService::from_ref(&state);

    // Establecer el usuario_id del usuario autenticado
    payload.usuario_id = Some(auth_user.user_id);

    let reporte = service.crear_soporte(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": serialize_soporte(&reporte),
            "message": "Reporte creado exitosamente"
        })),
    ))
}

/// Obtener un reporte por ID
pub async fn obtener_soporte(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let service = SoporteService::from_ref(&state);

    let reporte = service.obtener_por_id(id).await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_soporte(&reporte)
    })))
}

/// Listar soportes con filtros
pub async fn listar_soportes(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListarSoportesQuery>,
) -> Result<Json<ListaSoportesResponse>, AppError> {
    let service = SoporteService::from_ref(&state);

    let (soportes, total) = service
        .listar_soportes(
            params.estado,
            params.tipo,
            params.prioridad,
            params.usuario_id,
            params.responsable_id,
            params.limit,
            params.offset,
        )
        .await?;

    let soportes_json: Vec<Value> = soportes.iter().map(serialize_soporte).collect();

    Ok(Json(ListaSoportesResponse {
        soportes: soportes_json,
        total,
    }))
}

/// Actualizar un reporte
pub async fn actualizar_soporte(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<ActualizarSoporteError>,
) -> Result<Json<Value>, AppError> {
    let service = SoporteService::from_ref(&state);

    let usuario_accion = Some(auth_user.user_id);

    let reporte = service
        .actualizar_soporte(id, payload, usuario_accion)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_soporte(&reporte),
        "message": "Reporte actualizado exitosamente"
    })))
}

/// Eliminar un reporte
pub async fn eliminar_soporte(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let service = SoporteService::from_ref(&state);

    service.eliminar_soporte(id).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Reporte eliminado exitosamente"
    })))
}

// ==================== HANDLERS - LÓGICA DE NEGOCIO ====================

/// Cambiar estado de un soporte
pub async fn cambiar_estado_soporte(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(mut payload): Json<CambiarEstadoSoporte>,
) -> Result<Json<Value>, AppError> {
    let service = SoporteService::from_ref(&state);

    payload.usuario_id = Some(auth_user.user_id);

    let reporte = service.cambiar_estado(id, payload).await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_soporte(&reporte),
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
    let service = SoporteService::from_ref(&state);

    let usuario_accion = Some(auth_user.user_id);

    let reporte = service
        .asignar_responsable(id, payload, usuario_accion)
        .await?;

    Ok(Json(json!({
        "success": true,
        "data": serialize_soporte(&reporte),
        "message": "Responsable asignado exitosamente"
    })))
}

// ==================== HANDLERS - SEGUIMIENTO ====================

/// Agregar comentario/seguimiento a un reporte
pub async fn crear_seguimiento(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(reporte_id): Path<i32>,
    Json(mut payload): Json<NuevoSeguimientoSoporte>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = SoporteService::from_ref(&state);

    // Asignar el reporte_id de la URL y el usuario_id del autenticado
    payload.soporte_id = reporte_id;
    payload.usuario_id = Some(auth_user.user_id);

    // Si no se especifica tipo, usar Comentario por defecto
    if matches!(payload.tipo, TipoSeguimientoSoporte::Otro) && payload.estado_nuevo.is_none() {
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
    Query(params): Query<ListarSoportesQuery>,
) -> Result<Json<SeguimientosResponse>, AppError> {
    let service = SoporteService::from_ref(&state);

    let seguimientos = service
        .obtener_seguimientos(reporte_id, params.limit)
        .await?;

    let seguimientos_json: Vec<Value> = seguimientos.iter().map(serialize_seguimiento).collect();

    Ok(Json(SeguimientosResponse {
        seguimientos: seguimientos_json,
    }))
}

// ==================== HANDLERS - ESTADÍSTICAS ====================

/// Obtener estadísticas de soportes
pub async fn obtener_estadisticas(
    _auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let service = SoporteService::from_ref(&state);

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

/// Obtener soportes del usuario actual
pub async fn mis_soportes(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListarSoportesQuery>,
) -> Result<Json<ListaSoportesResponse>, AppError> {
    let service = SoporteService::from_ref(&state);

    // Extraer usuario_id del token JWT
    // Asumiendo que AuthUser tiene el user_id
    let usuario_id = extract_user_id(&auth_user);

    let (soportes, total) = service
        .listar_soportes(
            params.estado,
            params.tipo,
            params.prioridad,
            usuario_id,
            params.responsable_id,
            params.limit,
            params.offset,
        )
        .await?;

    let soportes_json: Vec<Value> = soportes.iter().map(serialize_soporte).collect();

    Ok(Json(ListaSoportesResponse {
        soportes: soportes_json,
        total,
    }))
}

/// Reportes asignados al usuario (para responsables)
pub async fn soportes_asignados(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListarSoportesQuery>,
) -> Result<Json<ListaSoportesResponse>, AppError> {
    let service = SoporteService::from_ref(&state);

    // Extraer usuario_id del token JWT
    let responsable_id = extract_user_id(&auth_user);

    let (soportes, total) = service
        .listar_soportes(
            params.estado,
            params.tipo,
            params.prioridad,
            params.usuario_id,
            responsable_id,
            params.limit,
            params.offset,
        )
        .await?;

    let soportes_json: Vec<Value> = soportes.iter().map(serialize_soporte).collect();

    Ok(Json(ListaSoportesResponse {
        soportes: soportes_json,
        total,
    }))
}

// Helper para extraer user_id del AuthUser
fn extract_user_id(auth_user: &AuthUser) -> Option<i32> {
    Some(auth_user.user_id)
}