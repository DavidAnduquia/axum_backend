use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    ModelTrait, Set,
};

use crate::{
    models::portafolio_contenido::{self, Entity as PortafolioContenido, Model as PortafolioContenidoModel},
    models::portafolio_contenido::{ActualizarPortafolioContenido, NuevoPortafolioContenido},
    utils::errors::AppError,
};

#[derive(Debug, Clone)]
pub struct PortafolioContenidoService {
    db: DatabaseConnection,
}

impl PortafolioContenidoService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn crear_contenido(
        &self,
        nuevo_contenido: NuevoPortafolioContenido,
    ) -> Result<PortafolioContenidoModel, AppError> {
        if nuevo_contenido.titulo.trim().is_empty() {
            return Err(AppError::BadRequest("El título es obligatorio".into()));
        }

        if nuevo_contenido.contenido.trim().is_empty() {
            return Err(AppError::BadRequest("El contenido es obligatorio".into()));
        }

        let ahora = Utc::now();
        let contenido = portafolio_contenido::ActiveModel {
            portafolio_id: Set(nuevo_contenido.portafolio_id),
            tipo_contenido: Set(nuevo_contenido.tipo_contenido),
            titulo: Set(nuevo_contenido.titulo),
            descripcion: Set(nuevo_contenido.descripcion),
            contenido: Set(nuevo_contenido.contenido),
            orden: Set(nuevo_contenido.orden),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let contenido_creado = contenido.insert(&self.db).await?;
        Ok(contenido_creado)
    }

    pub async fn obtener_contenidos(
        &self,
        portafolio_id: Option<i32>,
    ) -> Result<Vec<PortafolioContenidoModel>, DbErr> {
        let mut query = PortafolioContenido::find();

        if let Some(id) = portafolio_id {
            query = query.filter(portafolio_contenido::Column::PortafolioId.eq(id));
        }

        query
            .order_by_asc(portafolio_contenido::Column::Orden)
            .all(&self.db)
            .await
    }

    pub async fn obtener_contenido_por_id(
        &self,
        id: i32,
    ) -> Result<Option<PortafolioContenidoModel>, DbErr> {
        PortafolioContenido::find_by_id(id).one(&self.db).await
    }

    pub async fn actualizar_contenido(
        &self,
        id: i32,
        datos_actualizados: ActualizarPortafolioContenido,
    ) -> Result<PortafolioContenidoModel, AppError> {
        let contenido = PortafolioContenido::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".into()))?;

        let mut contenido: portafolio_contenido::ActiveModel = contenido.into();
        let ahora = Utc::now();

        if let Some(tipo) = datos_actualizados.tipo_contenido {
            contenido.tipo_contenido = Set(tipo);
        }

        if let Some(titulo) = datos_actualizados.titulo {
            if titulo.trim().is_empty() {
                return Err(AppError::BadRequest("El título no puede estar vacío".into()));
            }
            contenido.titulo = Set(titulo);
        }

        if let Some(descripcion) = datos_actualizados.descripcion {
            contenido.descripcion = Set(Some(descripcion));
        }

        if let Some(contenido_valor) = datos_actualizados.contenido {
            if contenido_valor.trim().is_empty() {
                return Err(AppError::BadRequest("El contenido no puede estar vacío".into()));
            }
            contenido.contenido = Set(contenido_valor);
        }

        if let Some(orden) = datos_actualizados.orden {
            contenido.orden = Set(orden);
        }

        contenido.updated_at = Set(Some(ahora));
        let contenido_actualizado = contenido.update(&self.db).await?;

        Ok(contenido_actualizado)
    }

    pub async fn eliminar_contenido(&self, id: i32) -> Result<(), AppError> {
        let contenido = PortafolioContenido::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".into()))?;

        contenido.delete(&self.db).await?;
        Ok(())
    }
}
