use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::{
    models::contenido_transversal::{self, Entity as ContenidoTransversal, Model as ContenidoTransversalModel},
    utils::errors::AppError,
};

pub use crate::models::contenido_transversal::{ActualizarContenido, NuevoContenido};

#[derive(Debug, Clone)]
pub struct ContenidoTransversalService {
    db: DatabaseConnection,
}

impl ContenidoTransversalService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn obtener_contenidos(&self) -> Result<Vec<ContenidoTransversalModel>, DbErr> {
        ContenidoTransversal::find().all(&self.db).await
    }

    pub async fn crear_contenido(
        &self,
        nuevo_contenido: NuevoContenido,
    ) -> Result<ContenidoTransversalModel, AppError> {
        if nuevo_contenido.nombre.trim().is_empty() {
            return Err(AppError::BadRequest("El nombre es obligatorio".to_string()));
        }

        let ahora = Utc::now();
        let contenido = contenido_transversal::ActiveModel {
            nombre: Set(nuevo_contenido.nombre),
            descripcion: Set(nuevo_contenido.descripcion),
            area_conocimiento_id: Set(nuevo_contenido.area_conocimiento_id),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let contenido_creado = contenido.insert(&self.db).await?;
        Ok(contenido_creado)
    }

    pub async fn actualizar_contenido(
        &self,
        id: i32,
        datos_actualizados: ActualizarContenido,
    ) -> Result<ContenidoTransversalModel, AppError> {
        let contenido = ContenidoTransversal::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".to_string()))?;

        let mut contenido: contenido_transversal::ActiveModel = contenido.into();

        if let Some(nombre) = datos_actualizados.nombre {
            contenido.nombre = Set(nombre);
        }
        if let Some(descripcion) = datos_actualizados.descripcion {
            contenido.descripcion = Set(Some(descripcion));
        }
        if let Some(area_id) = datos_actualizados.area_conocimiento_id {
            contenido.area_conocimiento_id = Set(area_id);
        }

        contenido.updated_at = Set(Some(Utc::now()));
        let contenido_actualizado = contenido.update(&self.db).await?;
        Ok(contenido_actualizado)
    }

    pub async fn eliminar_contenido(&self, id: i32) -> Result<(), AppError> {
        let contenido = ContenidoTransversal::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".to_string()))?;

        contenido.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<ContenidoTransversalModel> for ContenidoTransversalService {
    async fn get_all(&self) -> Result<Vec<ContenidoTransversalModel>, AppError> {
        self.obtener_contenidos().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<ContenidoTransversalModel>, AppError> {
        ContenidoTransversal::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, data: ContenidoTransversalModel) -> Result<ContenidoTransversalModel, AppError> {
        self.crear_contenido(NuevoContenido {
            nombre: data.nombre,
            descripcion: data.descripcion,
            area_conocimiento_id: data.area_conocimiento_id,
        }).await
    }

    async fn update(&self, id: i32, data: ContenidoTransversalModel) -> Result<ContenidoTransversalModel, AppError> {
        self.actualizar_contenido(id, ActualizarContenido {
            nombre: Some(data.nombre),
            descripcion: data.descripcion,
            area_conocimiento_id: Some(data.area_conocimiento_id),
        }).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_contenido(id).await
    }
}
