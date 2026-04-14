use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};

use crate::{
    models::calificacion::{self, Entity as Calificacion, Model as CalificacionModel},
    utils::errors::AppError,
};

pub use crate::models::calificacion::{ActualizarCalificacion, NuevaCalificacion};

#[derive(Debug, Clone)]
pub struct CalificacionService {
    db: DatabaseConnection,
}

impl CalificacionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn obtener_calificaciones(&self) -> Result<Vec<CalificacionModel>, DbErr> {
        Calificacion::find().all(&self.db).await
    }

    pub async fn obtener_calificaciones_por_estudiante(
        &self,
        estudiante_id: i64,
    ) -> Result<Vec<CalificacionModel>, DbErr> {
        Calificacion::find()
            .filter(calificacion::Column::EstudianteId.eq(estudiante_id))
            .all(&self.db)
            .await
    }

    pub async fn obtener_calificaciones_por_actividad(
        &self,
        actividad_id: i32,
    ) -> Result<Vec<CalificacionModel>, DbErr> {
        Calificacion::find()
            .filter(calificacion::Column::ActividadId.eq(actividad_id))
            .all(&self.db)
            .await
    }

    pub async fn crear_calificacion(
        &self,
        nueva_calificacion: NuevaCalificacion,
    ) -> Result<CalificacionModel, AppError> {
        let ahora = Utc::now();
        let calificacion = calificacion::ActiveModel {
            actividad_id: Set(nueva_calificacion.actividad_id),
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
    ) -> Result<CalificacionModel, AppError> {
        let calificacion = Calificacion::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Calificación no encontrada".to_string()))?;

        let mut calificacion: calificacion::ActiveModel = calificacion.into();

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
        let calificacion = Calificacion::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Calificación no encontrada".to_string()))?;

        calificacion.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<CalificacionModel> for CalificacionService {
    async fn get_all(&self) -> Result<Vec<CalificacionModel>, AppError> {
        self.obtener_calificaciones().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<CalificacionModel>, AppError> {
        Calificacion::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, data: CalificacionModel) -> Result<CalificacionModel, AppError> {
        self.crear_calificacion(NuevaCalificacion {
            actividad_id: data.actividad_id,
            estudiante_id: data.estudiante_id,
            calificacion: data.calificacion,
            retroalimentacion: data.retroalimentacion,
        }).await
    }

    async fn update(&self, id: i32, data: CalificacionModel) -> Result<CalificacionModel, AppError> {
        self.actualizar_calificacion(id, ActualizarCalificacion {
            calificacion: Some(data.calificacion),
            retroalimentacion: data.retroalimentacion,
        }).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_calificacion(id).await
    }
}
