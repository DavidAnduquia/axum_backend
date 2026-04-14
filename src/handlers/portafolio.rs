use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    middleware::auth::AuthUser,
    models::{AppState, portafolio::Model as PortafolioModel},
    services::{
        personalizacion_portafolio_service::PersonalizacionPortafolioService,
        portafolio_service::{PortafolioService, NuevoPortafolio},
    },
    utils::errors::AppError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortadaCursoDesignDto {
    pub titulo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitulo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descripcion: Option<String>,
    #[serde(default)]
    pub bloques: Vec<PortadaBlockDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortadaBlockDto {
    pub kind: String, // "texto" | "imagen"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortadaCursoPayload {
    pub estilos: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortadaCursoResponse {
    pub portafolio: PortafolioModel,
    pub estilos: Option<Value>,
}

fn get_db_connection(state: &AppState) -> Result<sea_orm::DatabaseConnection, AppError> {
    let executor = state
        .db
        .clone()
        .ok_or_else(|| AppError::ServiceUnavailable("Database connection is not available".into()))?;
    Ok(executor.connection())
}

async fn get_or_create_portafolio(
    curso_id: i32,
    _auth_user: &AuthUser,
    state: &AppState,
) -> Result<PortafolioModel, AppError> {
    let db = get_db_connection(state)?;
    let service = PortafolioService::global(&db);

    // Buscar portafolios existentes para el curso
    let mut portafolios = service
        .obtener_portafolios_por_curso(curso_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string().into()))?;

    if let Some(port) = portafolios.pop() {
        return Ok(port);
    }

    // Crear un portafolio básico para el curso
    let nuevo = NuevoPortafolio {
        curso_id: Some(curso_id),
        nombre: format!("Portafolio del curso {}", curso_id),
        descripcion: Some("Portafolio principal del curso".to_string()),
    };

    let creado = service.crear_portafolio(nuevo).await?;
    Ok(creado)
}

pub async fn obtener_portada_curso(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(curso_id): Path<i32>,
) -> Result<Json<PortadaCursoResponse>, AppError> {
    let db = get_db_connection(&state)?;
    let portafolio = get_or_create_portafolio(curso_id, &auth_user, &state).await?;

    let personalizacion_service = PersonalizacionPortafolioService::new(db.clone());
    let personalizacion = personalizacion_service
        .obtener_personalizacion_por_portafolio(portafolio.id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string().into()))?;

    let estilos = personalizacion.and_then(|p| p.estilos);

    Ok(Json(PortadaCursoResponse { portafolio, estilos }))
}

pub async fn guardar_portada_curso(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(curso_id): Path<i32>,
    Json(payload): Json<PortadaCursoPayload>,
) -> Result<Json<PortadaCursoResponse>, AppError> {
    let db = get_db_connection(&state)?;
    let portafolio = get_or_create_portafolio(curso_id, &auth_user, &state).await?;

    let servicio = PersonalizacionPortafolioService::new(db.clone());

    // Intentar obtener personalización existente
    let existente = servicio
        .obtener_personalizacion_por_portafolio(portafolio.id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string().into()))?;

    let estilos_value = Some(payload.estilos);

    use crate::services::personalizacion_portafolio_service::{ActualizarPersonalizacion, NuevaPersonalizacion};

    if let Some(actual) = existente {
        let actualizada = servicio
            .actualizar_personalizacion(
                actual.id,
                ActualizarPersonalizacion {
                    estilos: estilos_value.clone(),
                    orden_componentes: None,
                    privacidad_componentes: None,
                },
            )   
            .await?;

        Ok(Json(PortadaCursoResponse {
            portafolio,
            estilos: actualizada.estilos,
        }))
    } else {
        let creada = servicio
            .crear_personalizacion(NuevaPersonalizacion {
                portafolio_id: portafolio.id,
                estilos: estilos_value.clone(),
                orden_componentes: None,
                privacidad_componentes: None,
            })
            .await?;

        Ok(Json(PortadaCursoResponse {
            portafolio,
            estilos: creada.estilos,
        }))
    }
}