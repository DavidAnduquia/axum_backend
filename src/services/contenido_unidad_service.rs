use axum::extract::FromRef;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, Order,
    QueryFilter, QueryOrder, Set,
};

use crate::{
    database::DbExecutor,
    models::{
        contenido_unidad::{self, Entity as Contenido, Model as ContenidoModel},
        AppState,
    },
    utils::errors::AppError,
};

pub use crate::models::contenido_unidad::{ActualizarContenidoUnidad, NuevoContenidoUnidad};

#[derive(Debug, Clone)]
pub struct ContenidoUnidadService {
    db: DbExecutor,
}

impl ContenidoUnidadService {
    pub fn new(db: DbExecutor) -> Self {
        Self { db }
    }

    fn connection(&self) -> DatabaseConnection {
        self.db.connection()
    }

    pub async fn crear_contenido(
        &self,
        nuevo: NuevoContenidoUnidad,
    ) -> Result<ContenidoModel, AppError> {
        if nuevo.titulo.trim().is_empty() {
            return Err(AppError::BadRequest("El ttulo es obligatorio".into()));
        }

        let db = self.connection();

        // Dejar que la BD autoincremente el id (no establecerlo manualmente)
        let contenido = contenido_unidad::ActiveModel {
            unidad_id: Set(nuevo.unidad_id),
            tipo_contenido: Set(nuevo.tipo_contenido),
            titulo: Set(nuevo.titulo),
            descripcion: Set(nuevo.descripcion),
            orden: Set(nuevo.orden),
            contenido: Set(nuevo.contenido),
            url: Set(nuevo.url),
            visible: Set(nuevo.visible),
            obligatorio: Set(nuevo.obligatorio),
            puntos: Set(nuevo.puntos),
            fecha_limite: Set(None),
            duracion_estimada: Set(None),
            examen_id: Set(None),
            entrega_id: Set(None),
            fecha_creacion: Set(None),
            fecha_actualizacion: Set(None),
            ..Default::default()
        };

        let creado = contenido.insert(&db).await?;
        Ok(creado)
    }

    pub async fn obtener_contenidos_por_unidad(
        &self,
        unidad_id: i32,
    ) -> Result<Vec<ContenidoModel>, DbErr> {
        let db = self.connection();
        Contenido::find()
            .filter(contenido_unidad::Column::UnidadId.eq(unidad_id))
            .order_by(contenido_unidad::Column::Orden, Order::Asc)
            .all(&db)
            .await
    }

    pub async fn obtener_contenido_por_id(
        &self,
        id: i32,
    ) -> Result<Option<ContenidoModel>, DbErr> {
        let db = self.connection();
        Contenido::find_by_id(id).one(&db).await
    }

    pub async fn actualizar_contenido(
        &self,
        id: i32,
        datos: ActualizarContenidoUnidad,
    ) -> Result<ContenidoModel, AppError> {
        let db = self.connection();
        let contenido = Contenido::find_by_id(id)
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".into()))?;

        let mut contenido: contenido_unidad::ActiveModel = contenido.into();

        if let Some(tipo) = datos.tipo_contenido {
            contenido.tipo_contenido = Set(tipo);
        }

        if let Some(titulo) = datos.titulo {
            if titulo.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "El ttulo no puede estar vaco".into(),
                ));
            }
            contenido.titulo = Set(titulo);
        }

        if let Some(descripcion) = datos.descripcion {
            contenido.descripcion = Set(Some(descripcion));
        }

        if let Some(texto) = datos.contenido {
            contenido.contenido = Set(Some(texto));
        }

        if let Some(url) = datos.url {
            contenido.url = Set(Some(url));
        }

        if let Some(orden) = datos.orden {
            contenido.orden = Set(orden);
        }

        if let Some(visible) = datos.visible {
            contenido.visible = Set(visible);
        }

        if let Some(obligatorio) = datos.obligatorio {
            contenido.obligatorio = Set(obligatorio);
        }

        if let Some(puntos) = datos.puntos {
            contenido.puntos = Set(Some(puntos));
        }

        let actualizado = contenido.update(&db).await?;
        Ok(actualizado)
    }

    pub async fn eliminar_contenido(&self, id: i32) -> Result<(), AppError> {
        let db = self.connection();
        let contenido = Contenido::find_by_id(id)
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("Contenido no encontrado".into()))?;

        contenido.delete(&db).await?;
        Ok(())
    }
}

impl FromRef<AppState> for ContenidoUnidadService {
    fn from_ref(state: &AppState) -> Self {
        let executor = state
            .db
            .clone()
            .expect("Database connection is not available");
        ContenidoUnidadService::new(executor)
    }
}
