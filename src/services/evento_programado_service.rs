use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set,
};

use crate::{
    models::evento_programado::{self, Entity as EventoProgramado, Model as EventoProgramadoModel},
    utils::errors::AppError,
};

pub use crate::models::evento_programado::{ActualizarEvento, NuevoEvento};

#[derive(Debug, Clone)]
pub struct EventoProgramadoService {
    db: DatabaseConnection,
}

impl EventoProgramadoService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn listar_eventos(&self) -> Result<Vec<EventoProgramadoModel>, DbErr> {
        EventoProgramado::find().all(&self.db).await
    }

    pub async fn crear_evento(
        &self,
        nuevo_evento: NuevoEvento,
    ) -> Result<EventoProgramadoModel, AppError> {
        if nuevo_evento.titulo.trim().is_empty() {
            return Err(AppError::BadRequest("El título es obligatorio".to_string()));
        }

        let ahora = Utc::now();
        let evento = evento_programado::ActiveModel {
            titulo: Set(nuevo_evento.titulo),
            descripcion: Set(nuevo_evento.descripcion),
            fecha_inicio: Set(nuevo_evento.fecha_inicio),
            fecha_fin: Set(nuevo_evento.fecha_fin),
            tipo_evento: Set(nuevo_evento.tipo_evento),
            curso_id: Set(nuevo_evento.curso_id),
            profesor_id: Set(nuevo_evento.profesor_id),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let evento_creado = evento.insert(&self.db).await?;
        Ok(evento_creado)
    }

    pub async fn actualizar_evento(
        &self,
        id: i32,
        datos_actualizados: ActualizarEvento,
    ) -> Result<EventoProgramadoModel, AppError> {
        let evento = EventoProgramado::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Evento no encontrado".to_string()))?;

        let mut evento: evento_programado::ActiveModel = evento.into();

        if let Some(titulo) = datos_actualizados.titulo {
            evento.titulo = Set(titulo);
        }
        if let Some(descripcion) = datos_actualizados.descripcion {
            evento.descripcion = Set(Some(descripcion));
        }
        if let Some(fecha_inicio) = datos_actualizados.fecha_inicio {
            evento.fecha_inicio = Set(fecha_inicio);
        }
        if let Some(fecha_fin) = datos_actualizados.fecha_fin {
            evento.fecha_fin = Set(fecha_fin);
        }
        if let Some(tipo) = datos_actualizados.tipo_evento {
            evento.tipo_evento = Set(tipo);
        }

        evento.updated_at = Set(Some(Utc::now()));
        let evento_actualizado = evento.update(&self.db).await?;
        Ok(evento_actualizado)
    }

    pub async fn eliminar_evento(&self, id: i32) -> Result<(), AppError> {
        let evento = EventoProgramado::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Evento no encontrado".to_string()))?;

        evento.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<EventoProgramadoModel> for EventoProgramadoService {
    async fn get_all(&self) -> Result<Vec<EventoProgramadoModel>, AppError> {
        self.listar_eventos().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<EventoProgramadoModel>, AppError> {
        EventoProgramado::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, data: EventoProgramadoModel) -> Result<EventoProgramadoModel, AppError> {
        self.crear_evento(NuevoEvento {
            titulo: data.titulo,
            descripcion: data.descripcion,
            fecha_inicio: data.fecha_inicio,
            fecha_fin: data.fecha_fin,
            tipo_evento: data.tipo_evento,
            curso_id: data.curso_id,
            profesor_id: data.profesor_id,
        }).await
    }

    async fn update(&self, id: i32, data: EventoProgramadoModel) -> Result<EventoProgramadoModel, AppError> {
        self.actualizar_evento(id, ActualizarEvento {
            titulo: Some(data.titulo),
            descripcion: data.descripcion,
            fecha_inicio: Some(data.fecha_inicio),
            fecha_fin: Some(data.fecha_fin),
            tipo_evento: Some(data.tipo_evento),
        }).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_evento(id).await
    }
}
