use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use serde_json::Value;

use crate::{
    models::personalizacion_modulo::{self, Entity as PersonalizacionModulo, Model as PersonalizacionModuloModel},
    utils::errors::AppError,
};

pub use crate::models::personalizacion_modulo::{ActualizarPersonalizacionModulo, NuevaPersonalizacionModulo};

#[derive(Debug, Clone)]
pub struct PersonalizacionModuloService {
    db: DatabaseConnection,
}

impl PersonalizacionModuloService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn crear_personalizacion(
        &self,
        nueva_personalizacion: NuevaPersonalizacionModulo,
    ) -> Result<PersonalizacionModuloModel, AppError> {
        let ahora = Utc::now();
        let personalizacion = personalizacion_modulo::ActiveModel {
            modulo_id: Set(nueva_personalizacion.modulo_id),
            estilos: Set(nueva_personalizacion.estilos),
            orden_componentes: Set(nueva_personalizacion.orden_componentes),
            privacidad_componentes: Set(nueva_personalizacion.privacidad_componentes),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let personalizacion_creada = personalizacion.insert(&self.db).await?;
        Ok(personalizacion_creada)
    }

    pub async fn obtener_personalizaciones(
        &self,
    ) -> Result<Vec<PersonalizacionModuloModel>, DbErr> {
        PersonalizacionModulo::find()
            .all(&self.db)
            .await
    }

    pub async fn obtener_personalizacion_por_id(
        &self,
        id: i32,
    ) -> Result<Option<PersonalizacionModuloModel>, DbErr> {
        PersonalizacionModulo::find_by_id(id).one(&self.db).await
    }

    pub async fn obtener_personalizacion_por_modulo(
        &self,
        modulo_id: i32,
    ) -> Result<Option<PersonalizacionModuloModel>, DbErr> {
        PersonalizacionModulo::find()
            .filter(personalizacion_modulo::Column::ModuloId.eq(modulo_id))
            .one(&self.db)
            .await
    }

    pub async fn actualizar_personalizacion(
        &self,
        id: i32,
        datos_actualizados: ActualizarPersonalizacionModulo,
    ) -> Result<PersonalizacionModuloModel, AppError> {
        let personalizacion = PersonalizacionModulo::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Personalización no encontrada".to_string()))?;

        let mut personalizacion: personalizacion_modulo::ActiveModel = personalizacion.into();
        let ahora = Utc::now();

        if let Some(estilos) = datos_actualizados.estilos {
            personalizacion.estilos = Set(Some(estilos));
        }

        if let Some(orden) = datos_actualizados.orden_componentes {
            personalizacion.orden_componentes = Set(Some(orden));
        }

        if let Some(privacidad) = datos_actualizados.privacidad_componentes {
            personalizacion.privacidad_componentes = Set(Some(privacidad));
        }

        personalizacion.updated_at = Set(Some(ahora));
        let personalizacion_actualizada = personalizacion.update(&self.db).await?;

        Ok(personalizacion_actualizada)
    }

    pub async fn eliminar_personalizacion(&self, id: i32) -> Result<(), AppError> {
        let personalizacion = PersonalizacionModulo::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Personalización no encontrada".to_string()))?;

        personalizacion.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<PersonalizacionModuloModel> for PersonalizacionModuloService {
    async fn get_all(&self) -> Result<Vec<PersonalizacionModuloModel>, AppError> {
        self.obtener_personalizaciones().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<PersonalizacionModuloModel>, AppError> {
        self.obtener_personalizacion_por_id(id).await.map_err(Into::into)
    }

    async fn create(&self, data: PersonalizacionModuloModel) -> Result<PersonalizacionModuloModel, AppError> {
        self.crear_personalizacion(NuevaPersonalizacionModulo {
            modulo_id: data.modulo_id,
            estilos: data.estilos,
            orden_componentes: data.orden_componentes,
            privacidad_componentes: data.privacidad_componentes,
        })
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: PersonalizacionModuloModel,
    ) -> Result<PersonalizacionModuloModel, AppError> {
        self.actualizar_personalizacion(
            id,
            ActualizarPersonalizacionModulo {
                estilos: data.estilos,
                orden_componentes: data.orden_componentes,
                privacidad_componentes: data.privacidad_componentes,
            },
        )
        .await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_personalizacion(id).await
    }
}
