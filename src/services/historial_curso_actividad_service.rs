use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::{
    models::historial_curso_actividad::{self, Entity as HistorialCursoActividad, Model as HistorialCursoActividadModel},
    utils::errors::AppError,
};

pub use crate::models::historial_curso_actividad::{ActualizarActividadHistorial, NuevaActividadHistorial};

#[derive(Debug, Clone)]
pub struct HistorialCursoActividadService {
    db: DatabaseConnection,
}

impl HistorialCursoActividadService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn crear_actividad_historial(
        &self,
        nueva_actividad: NuevaActividadHistorial,
    ) -> Result<HistorialCursoActividadModel, AppError> {
        let ahora = Utc::now();
        let fecha_completado = if nueva_actividad.completado {
            Some(ahora)
        } else {
            None
        };

        let actividad = historial_curso_actividad::ActiveModel {
            historial_curso_id: Set(nueva_actividad.historial_curso_id),
            actividad_id: Set(nueva_actividad.actividad_id),
            calificacion: Set(nueva_actividad.calificacion),
            completado: Set(nueva_actividad.completado),
            fecha_completado: Set(fecha_completado),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let actividad_creada = actividad.insert(&self.db).await?;
        Ok(actividad_creada)
    }

    pub async fn obtener_actividades_historial(
        &self,
    ) -> Result<Vec<HistorialCursoActividadModel>, DbErr> {
        HistorialCursoActividad::find()
            .all(&self.db)
            .await
    }

    pub async fn obtener_actividad_por_ids(
        &self,
        historial_curso_id: i32,
        actividad_id: i32,
    ) -> Result<Option<HistorialCursoActividadModel>, DbErr> {
        HistorialCursoActividad::find()
            .filter(historial_curso_actividad::Column::HistorialCursoId.eq(historial_curso_id))
            .filter(historial_curso_actividad::Column::ActividadId.eq(actividad_id))
            .one(&self.db)
            .await
    }

    pub async fn actualizar_actividad_historial(
        &self,
        historial_curso_id: i32,
        actividad_id: i32,
        datos_actualizados: ActualizarActividadHistorial,
    ) -> Result<HistorialCursoActividadModel, AppError> {
        let actividad = HistorialCursoActividad::find()
            .filter(historial_curso_actividad::Column::HistorialCursoId.eq(historial_curso_id))
            .filter(historial_curso_actividad::Column::ActividadId.eq(actividad_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Actividad no encontrada".to_string()))?;

        let mut actividad: historial_curso_actividad::ActiveModel = actividad.into();
        let ahora = Utc::now();

        if let Some(calificacion) = datos_actualizados.calificacion {
            actividad.calificacion = Set(Some(calificacion));
        }

        if let Some(completado) = datos_actualizados.completado {
            actividad.completado = Set(completado);
            actividad.fecha_completado = Set(if completado { Some(ahora) } else { None });
        }

        actividad.updated_at = Set(Some(ahora));
        let actividad_actualizada = actividad.update(&self.db).await?;

        Ok(actividad_actualizada)
    }

    pub async fn eliminar_actividad_historial(
        &self,
        historial_curso_id: i32,
        actividad_id: i32,
    ) -> Result<(), AppError> {
        let actividad = HistorialCursoActividad::find()
            .filter(historial_curso_actividad::Column::HistorialCursoId.eq(historial_curso_id))
            .filter(historial_curso_actividad::Column::ActividadId.eq(actividad_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Actividad no encontrada".to_string()))?;

        actividad.delete(&self.db).await?;
        Ok(())
    }

    pub async fn obtener_actividades_por_historial(
        &self,
        historial_curso_id: i32,
    ) -> Result<Vec<HistorialCursoActividadModel>, DbErr> {
        HistorialCursoActividad::find()
            .filter(historial_curso_actividad::Column::HistorialCursoId.eq(historial_curso_id))
            .all(&self.db)
            .await
    }

    pub async fn actualizar_estados_actividades(&self) -> Result<u64, AppError> {
        let ahora = Utc::now();
        let result = HistorialCursoActividad::update_many()
            .col_expr(historial_curso_actividad::Column::Completado, Expr::value(true))
            .col_expr(historial_curso_actividad::Column::FechaCompletado, Expr::value(ahora))
            .filter(historial_curso_actividad::Column::Completado.eq(false))
            .filter(historial_curso_actividad::Column::FechaCompletado.is_null())
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}

#[async_trait]
impl crate::traits::service::CrudService<HistorialCursoActividadModel> for HistorialCursoActividadService {
    async fn get_all(&self) -> Result<Vec<HistorialCursoActividadModel>, AppError> {
        self.obtener_actividades_historial().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<HistorialCursoActividadModel>, AppError> {
        HistorialCursoActividad::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn create(&self, data: HistorialCursoActividadModel) -> Result<HistorialCursoActividadModel, AppError> {
        self.crear_actividad_historial(NuevaActividadHistorial {
            historial_curso_id: data.historial_curso_id,
            actividad_id: data.actividad_id,
            calificacion: data.calificacion,
            completado: data.completado,
        })
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: HistorialCursoActividadModel,
    ) -> Result<HistorialCursoActividadModel, AppError> {
        let actividad = HistorialCursoActividad::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Actividad no encontrada".to_string()))?;

        let mut actividad: historial_curso_actividad::ActiveModel = actividad.into();
        let ahora = Utc::now();

        actividad.calificacion = Set(data.calificacion);
        actividad.completado = Set(data.completado);
        actividad.fecha_completado = Set(if data.completado { Some(ahora) } else { None });
        actividad.updated_at = Set(Some(ahora));

        actividad.update(&self.db).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        let actividad = HistorialCursoActividad::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Actividad no encontrada".to_string()))?;

        actividad.delete(&self.db).await?;
        Ok(())
    }
}
