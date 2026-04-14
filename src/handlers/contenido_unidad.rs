use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    middleware::auth::AuthUser,
    models::{contenido_unidad::Model as ContenidoModel, AppState},
    services::contenido_unidad_service::{
        ActualizarContenidoUnidad, ContenidoUnidadService, NuevoContenidoUnidad,
    },
    utils::errors::AppError,
};

pub async fn crear_contenido(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<NuevoContenidoUnidad>,
) -> Result<(StatusCode, Json<ContenidoModel>), AppError> {
    let service = ContenidoUnidadService::from_ref(&state);
    let contenido = service.crear_contenido(payload).await?;
    Ok((StatusCode::CREATED, Json(contenido)))
}

pub async fn listar_contenidos_por_unidad(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(unidad_id): Path<i32>,
) -> Result<Json<Vec<ContenidoModel>>, AppError> {
    let service = ContenidoUnidadService::from_ref(&state);
    let contenidos = service
        .obtener_contenidos_por_unidad(unidad_id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(contenidos))
}

pub async fn obtener_contenido(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ContenidoModel>, AppError> {
    let service = ContenidoUnidadService::from_ref(&state);
    match service
        .obtener_contenido_por_id(id)
        .await
        .map_err(AppError::from)?
    {
        Some(contenido) => Ok(Json(contenido)),
        None => Err(AppError::NotFound(
            format!("Contenido {} no encontrado", id).into(),
        )),
    }
}

pub async fn actualizar_contenido(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<ActualizarContenidoUnidad>,
) -> Result<Json<ContenidoModel>, AppError> {
    let service = ContenidoUnidadService::from_ref(&state);
    let contenido = service.actualizar_contenido(id, payload).await?;
    Ok(Json(contenido))
}

pub async fn eliminar_contenido(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let service = ContenidoUnidadService::from_ref(&state);
    service.eliminar_contenido(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
