use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};

use crate::{
    models::plantilla_curso::{self, Entity as PlantillaCurso, Model as PlantillaCursoModel},
    utils::errors::AppError,
};

pub use crate::models::plantilla_curso::{ActualizarPlantillaCurso, NuevaPlantillaCurso};

#[derive(Debug, Clone)]
pub struct PlantillaCursoService {
    db: DatabaseConnection,
}

impl PlantillaCursoService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn obtener_plantillas(&self) -> Result<Vec<PlantillaCursoModel>, DbErr> {
        PlantillaCurso::find()
            .all(&self.db)
            .await
    }

    pub async fn crear_plantilla(
        &self,
        nueva_plantilla: NuevaPlantillaCurso,
    ) -> Result<PlantillaCursoModel, AppError> {
        if nueva_plantilla.nombre.trim().is_empty() {
            return Err(AppError::BadRequest("El nombre es obligatorio".to_string()));
        }

        let ahora = Utc::now();
        let plantilla = plantilla_curso::ActiveModel {
            nombre: Set(nueva_plantilla.nombre),
            descripcion: Set(nueva_plantilla.descripcion),
            activa: Set(nueva_plantilla.activa),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let plantilla_creada = plantilla.insert(&self.db).await?;
        Ok(plantilla_creada)
    }

    pub async fn obtener_plantilla_por_id(
        &self,
        id: i32,
    ) -> Result<Option<PlantillaCursoModel>, DbErr> {
        PlantillaCurso::find_by_id(id).one(&self.db).await
    }

    pub async fn editar_plantilla(
        &self,
        id: i32,
        datos_actualizados: ActualizarPlantillaCurso,
    ) -> Result<PlantillaCursoModel, AppError> {
        let plantilla = PlantillaCurso::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Plantilla no encontrada".to_string()))?;

        let mut plantilla: plantilla_curso::ActiveModel = plantilla.into();
        let ahora = Utc::now();

        if let Some(nombre) = datos_actualizados.nombre {
            if nombre.trim().is_empty() {
                return Err(AppError::BadRequest("El nombre no puede estar vacío".to_string()));
            }
            plantilla.nombre = Set(nombre);
        }

        if let Some(descripcion) = datos_actualizados.descripcion {
            plantilla.descripcion = Set(Some(descripcion));
        }

        if let Some(activa) = datos_actualizados.activa {
            plantilla.activa = Set(activa);
        }

        plantilla.updated_at = Set(Some(ahora));
        let plantilla_actualizada = plantilla.update(&self.db).await?;

        Ok(plantilla_actualizada)
    }

    pub async fn eliminar_plantilla(&self, id: i32) -> Result<(), AppError> {
        let plantilla = PlantillaCurso::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Plantilla no encontrada".to_string()))?;

        plantilla.delete(&self.db).await?;
        Ok(())
    }

    // TODO: Implementar aplicar_plantilla_a_curso cuando existan los modelos relacionados
    // pub async fn aplicar_plantilla_a_curso(
    //     &self,
    //     plantilla_id: i32,
    //     curso_id: i32,
    // ) -> Result<(), AppError> {
    //     // Implementación pendiente
    // }
}

#[async_trait]
impl crate::traits::service::CrudService<PlantillaCursoModel> for PlantillaCursoService {
    async fn get_all(&self) -> Result<Vec<PlantillaCursoModel>, AppError> {
        self.obtener_plantillas().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<PlantillaCursoModel>, AppError> {
        self.obtener_plantilla_por_id(id).await.map_err(Into::into)
    }

    async fn create(&self, data: PlantillaCursoModel) -> Result<PlantillaCursoModel, AppError> {
        self.crear_plantilla(NuevaPlantillaCurso {
            nombre: data.nombre,
            descripcion: data.descripcion,
            activa: data.activa,
        })
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: PlantillaCursoModel,
    ) -> Result<PlantillaCursoModel, AppError> {
        self.editar_plantilla(
            id,
            ActualizarPlantillaCurso {
                nombre: Some(data.nombre),
                descripcion: data.descripcion,
                activa: Some(data.activa),
            },
        )
        .await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_plantilla(id).await
    }
}
