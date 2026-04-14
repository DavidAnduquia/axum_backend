use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::{
    models::evaluacion_sesion::{self, Entity as EvaluacionSesion, Model as EvaluacionSesionModel},
    utils::errors::AppError,
};

pub use crate::models::evaluacion_sesion::{ActualizarEvaluacion, NuevaEvaluacion};

#[derive(Debug, Clone)]
pub struct EvaluacionSesionService {
    db: DatabaseConnection,
}

impl EvaluacionSesionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn obtener_evaluaciones(&self) -> Result<Vec<EvaluacionSesionModel>, DbErr> {
        EvaluacionSesion::find().all(&self.db).await
    }

    pub async fn crear_evaluacion(
        &self,
        nueva_evaluacion: NuevaEvaluacion,
    ) -> Result<EvaluacionSesionModel, AppError> {
        if nueva_evaluacion.nombre.trim().is_empty() {
            return Err(AppError::bad_request("El nombre es obligatorio"));
        }

        let ahora = Utc::now();
        let evaluacion = evaluacion_sesion::ActiveModel {
            sesion_id: Set(nueva_evaluacion.sesion_id),
            nombre: Set(nueva_evaluacion.nombre),
            descripcion: Set(nueva_evaluacion.descripcion),
            fecha_inicio: Set(nueva_evaluacion.fecha_inicio),
            fecha_fin: Set(nueva_evaluacion.fecha_fin),
            tipo_evaluacion: Set(nueva_evaluacion.tipo_evaluacion),
            peso: Set(nueva_evaluacion.peso),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let evaluacion_creada = evaluacion.insert(&self.db).await?;
        Ok(evaluacion_creada)
    }

    pub async fn actualizar_evaluacion(
        &self,
        id: i32,
        datos_actualizados: ActualizarEvaluacion,
    ) -> Result<EvaluacionSesionModel, AppError> {
        let evaluacion = EvaluacionSesion::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Evaluación no encontrada"))?

        let mut evaluacion: evaluacion_sesion::ActiveModel = evaluacion.into();

        if let Some(nombre) = datos_actualizados.nombre {
            evaluacion.nombre = Set(nombre);
        }
        if let Some(descripcion) = datos_actualizados.descripcion {
            evaluacion.descripcion = Set(Some(descripcion));
        }
        if let Some(fecha_inicio) = datos_actualizados.fecha_inicio {
            evaluacion.fecha_inicio = Set(fecha_inicio);
        }
        if let Some(fecha_fin) = datos_actualizados.fecha_fin {
            evaluacion.fecha_fin = Set(fecha_fin);
        }
        if let Some(tipo) = datos_actualizados.tipo_evaluacion {
            evaluacion.tipo_evaluacion = Set(tipo);
        }
        if let Some(peso) = datos_actualizados.peso {
            evaluacion.peso = Set(peso);
        }

        evaluacion.updated_at = Set(Some(Utc::now()));
        let evaluacion_actualizada = evaluacion.update(&self.db).await?;
        Ok(evaluacion_actualizada)
    }

    pub async fn eliminar_evaluacion(&self, id: i32) -> Result<(), AppError> {
        let evaluacion = EvaluacionSesion::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Evaluación no encontrada"))?;

        evaluacion.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<EvaluacionSesionModel> for EvaluacionSesionService {
    async fn get_all(&self) -> Result<Vec<EvaluacionSesionModel>, AppError> {
        self.obtener_evaluaciones().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<EvaluacionSesionModel>, AppError> {
        EvaluacionSesion::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, data: EvaluacionSesionModel) -> Result<EvaluacionSesionModel, AppError> {
        self.crear_evaluacion(NuevaEvaluacion {
            sesion_id: data.sesion_id,
            nombre: data.nombre,
            descripcion: data.descripcion,
            fecha_inicio: data.fecha_inicio,
            fecha_fin: data.fecha_fin,
            tipo_evaluacion: data.tipo_evaluacion,
            peso: data.peso,
        }).await
    }

    async fn update(&self, id: i32, data: EvaluacionSesionModel) -> Result<EvaluacionSesionModel, AppError> {
        self.actualizar_evaluacion(id, ActualizarEvaluacion {
            nombre: Some(data.nombre),
            descripcion: data.descripcion,
            fecha_inicio: Some(data.fecha_inicio),
            fecha_fin: Some(data.fecha_fin),
            tipo_evaluacion: Some(data.tipo_evaluacion),
            peso: Some(data.peso),
        }).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_evaluacion(id).await
    }
}
