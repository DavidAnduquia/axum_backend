use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use serde_json::Value;

use crate::{
    models::personalizacion_examen::{self, Entity as PersonalizacionExamen, Model as PersonalizacionExamenModel},
    utils::errors::AppError,
};

pub use crate::models::personalizacion_examen::{ActualizarPersonalizacionExamen, NuevaPersonalizacionExamen};

#[derive(Debug, Clone)]
pub struct PersonalizacionExamenService {
    db: DatabaseConnection,
}

impl PersonalizacionExamenService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn crear_personalizacion(
        &self,
        nueva_personalizacion: NuevaPersonalizacionExamen,
    ) -> Result<PersonalizacionExamenModel, AppError> {
        let ahora = Utc::now();
        let personalizacion = personalizacion_examen::ActiveModel {
            examen_id: Set(nueva_personalizacion.examen_id),
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
    ) -> Result<Vec<PersonalizacionExamenModel>, DbErr> {
        PersonalizacionExamen::find()
            .all(&self.db)
            .await
    }

    pub async fn obtener_personalizacion_por_id(
        &self,
        id: i32,
    ) -> Result<Option<PersonalizacionExamenModel>, DbErr> {
        PersonalizacionExamen::find_by_id(id).one(&self.db).await
    }

    pub async fn obtener_personalizacion_por_examen(
        &self,
        examen_id: i32,
    ) -> Result<Option<PersonalizacionExamenModel>, DbErr> {
        PersonalizacionExamen::find()
            .filter(personalizacion_examen::Column::ExamenId.eq(examen_id))
            .one(&self.db)
            .await
    }

    pub async fn actualizar_personalizacion(
        &self,
        id: i32,
        datos_actualizados: ActualizarPersonalizacionExamen,
    ) -> Result<PersonalizacionExamenModel, AppError> {
        let personalizacion = PersonalizacionExamen::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Personalización no encontrada".to_string()))?;

        let mut personalizacion: personalizacion_examen::ActiveModel = personalizacion.into();
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
        let personalizacion = PersonalizacionExamen::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Personalización no encontrada".to_string()))?;

        personalizacion.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<PersonalizacionExamenModel> for PersonalizacionExamenService {
    async fn get_all(&self) -> Result<Vec<PersonalizacionExamenModel>, AppError> {
        self.obtener_personalizaciones().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<PersonalizacionExamenModel>, AppError> {
        self.obtener_personalizacion_por_id(id).await.map_err(Into::into)
    }

    async fn create(&self, data: PersonalizacionExamenModel) -> Result<PersonalizacionExamenModel, AppError> {
        self.crear_personalizacion(NuevaPersonalizacionExamen {
            examen_id: data.examen_id,
            estilos: data.estilos,
            orden_componentes: data.orden_componentes,
            privacidad_componentes: data.privacidad_componentes,
        })
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: PersonalizacionExamenModel,
    ) -> Result<PersonalizacionExamenModel, AppError> {
        self.actualizar_personalizacion(
            id,
            ActualizarPersonalizacionExamen {
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
