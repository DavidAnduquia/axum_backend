use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

use crate::{
    models::contenido_plantilla::{self, Entity as ContenidoPlantilla, Model as ContenidoPlantillaModel},
    utils::errors::AppError,
};

pub use crate::models::contenido_plantilla::{ActualizarContenidoPlantilla, NuevoContenidoPlantilla};

#[derive(Debug, Clone)]
pub struct ContenidoPlantillaService {
    db: DatabaseConnection,
}

impl ContenidoPlantillaService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn obtener_contenidos(&self) -> Result<Vec<ContenidoPlantillaModel>, DbErr> {
        ContenidoPlantilla::find().all(&self.db).await
    }

    pub async fn crear_contenido(
        &self,
        nuevo_contenido: NuevoContenidoPlantilla,
    ) -> Result<ContenidoPlantillaModel, AppError> {
        if nuevo_contenido.nombre.trim().is_empty() {
            return Err(AppError::BadRequest("El nombre es obligatorio".to_string()));
        }

        let ahora = Utc::now();
        let contenido = contenido_plantilla::ActiveModel {
            plantilla_curso_id: Set(nuevo_contenido.plantilla_curso_id),
            nombre: Set(nuevo_contenido.nombre),
            descripcion: Set(nuevo_contenido.descripcion),
            tipo_contenido: Set(nuevo_contenido.tipo_contenido),
            orden: Set(nuevo_contenido.orden),
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
        datos_actualizados: ActualizarContenidoPlantilla,
    ) -> Result<ContenidoPlantillaModel, AppError> {
        let contenido = ContenidoPlantilla::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".to_string()))?;

        let mut contenido: contenido_plantilla::ActiveModel = contenido.into();

        if let Some(nombre) = datos_actualizados.nombre {
            contenido.nombre = Set(nombre);
        }
        if let Some(descripcion) = datos_actualizados.descripcion {
            contenido.descripcion = Set(Some(descripcion));
        }
        if let Some(tipo) = datos_actualizados.tipo_contenido {
            contenido.tipo_contenido = Set(tipo);
        }
        if let Some(orden) = datos_actualizados.orden {
            contenido.orden = Set(orden);
        }

        contenido.updated_at = Set(Some(Utc::now()));
        let contenido_actualizado = contenido.update(&self.db).await?;
        Ok(contenido_actualizado)
    }

    pub async fn eliminar_contenido(&self, id: i32) -> Result<(), AppError> {
        let contenido = ContenidoPlantilla::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".to_string()))?;

        contenido.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<ContenidoPlantillaModel> for ContenidoPlantillaService {
    async fn get_all(&self) -> Result<Vec<ContenidoPlantillaModel>, AppError> {
        self.obtener_contenidos().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<ContenidoPlantillaModel>, AppError> {
        ContenidoPlantilla::find_by_id(id).one(&self.db).await.map_err(Into::into)
    }

    async fn create(&self, data: ContenidoPlantillaModel) -> Result<ContenidoPlantillaModel, AppError> {
        self.crear_contenido(NuevoContenidoPlantilla {
            plantilla_curso_id: data.plantilla_curso_id,
            nombre: data.nombre,
            descripcion: data.descripcion,
            tipo_contenido: data.tipo_contenido,
            orden: data.orden,
        }).await
    }

    async fn update(&self, id: i32, data: ContenidoPlantillaModel) -> Result<ContenidoPlantillaModel, AppError> {
        self.actualizar_contenido(id, ActualizarContenidoPlantilla {
            nombre: Some(data.nombre),
            descripcion: data.descripcion,
            tipo_contenido: Some(data.tipo_contenido),
            orden: Some(data.orden),
        }).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_contenido(id).await
    }
}
