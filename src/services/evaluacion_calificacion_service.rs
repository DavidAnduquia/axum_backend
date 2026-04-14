use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::{
    models::evaluacion_calificacion::{self, Entity as EvaluacionCalificacion, Model as EvaluacionCalificacionModel},
    utils::errors::AppError,
};

pub use crate::models::evaluacion_calificacion::{ActualizarCalificacion, NuevaCalificacion};

#[derive(Debug, Clone)]
pub struct EvaluacionCalificacionService {
    db: DatabaseConnection,
}

impl EvaluacionCalificacionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn obtener_calificaciones(&self) -> Result<Vec<EvaluacionCalificacionModel>, DbErr> {
        EvaluacionCalificacion::find().all(&self.db).await
    }

    pub async fn crear_calificacion(
        &self,
        nueva_calificacion: NuevaCalificacion,
    ) -> Result<EvaluacionCalificacionModel, AppError> {
        let ahora = Utc::now();
        let calificacion = evaluacion_calificacion::ActiveModel {
            evaluacion_id: Set(nueva_calificacion.evaluacion_id),
            estudiante_id: Set(nueva_calificacion.estudiante_id),
            calificacion: Set(nueva_calificacion.calificacion),
            retroalimentacion: Set(nueva_calificacion.retroalimentacion),
            fecha_calificacion: Set(ahora),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let calificacion_creada = calificacion.insert(&self.db).await?;
        Ok(calificacion_creada)
    }

    pub async fn actualizar_calificacion(
        &self,
        id: i32,
        datos_actualizados: ActualizarCalificacion,
    ) -> Result<EvaluacionCalificacionModel, AppError> {
        let calificacion = EvaluacionCalificacion::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Calificación no encontrada".to_string()))?;

        let mut calificacion: evaluacion_calificacion::ActiveModel = calificacion.into();

        if let Some(calif) = datos_actualizados.calificacion {
            calificacion.calificacion = Set(calif);
        }
        if let Some(retro) = datos_actualizados.retroalimentacion {
            calificacion.retroalimentacion = Set(Some(retro));
        }

        calificacion.updated_at = Set(Some(Utc::now()));
        let calificacion_actualizada = calificacion.update(&self.db).await?;
        Ok(calificacion_actualizada)
    }

    pub async fn eliminar_calificacion(&self, id: i32) -> Result<(), AppError> {
        let calificacion = EvaluacionCalificacion::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Calificación no encontrada".to_string()))?;

        calificacion.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<EvaluacionCalificacionModel> for EvaluacionCalificacionService {
    async fn get_all(&self) -> Result<Vec<EvaluacionCalificacionModel>, AppError> {
        self.obtener_calificaciones().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<EvaluacionCalificacionModel>, AppError> {
        EvaluacionCalificacion::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, data: EvaluacionCalificacionModel) -> Result<EvaluacionCalificacionModel, AppError> {
        self.crear_calificacion(NuevaCalificacion {
            evaluacion_id: data.evaluacion_id,
            estudiante_id: data.estudiante_id,
            calificacion: data.calificacion,
            retroalimentacion: data.retroalimentacion,
        }).await
    }

    async fn update(&self, id: i32, data: EvaluacionCalificacionModel) -> Result<EvaluacionCalificacionModel, AppError> {
        self.actualizar_calificacion(id, ActualizarCalificacion {
            calificacion: Some(data.calificacion),
            retroalimentacion: data.retroalimentacion,
        }).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_calificacion(id).await
    }
}
