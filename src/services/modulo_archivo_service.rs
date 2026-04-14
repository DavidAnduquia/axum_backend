use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::{
    models::modulo_archivo::{self, Entity as ModuloArchivo, Model as ModuloArchivoModel},
    utils::errors::AppError,
};

pub use crate::models::modulo_archivo::{ActualizarModuloArchivo, NuevoModuloArchivo};

#[derive(Debug, Clone)]
pub struct ModuloArchivoService {
    db: DatabaseConnection,
}

impl ModuloArchivoService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn crear_archivo(
        &self,
        nuevo_archivo: NuevoModuloArchivo,
    ) -> Result<ModuloArchivoModel, AppError> {
        // Validaciones
        if nuevo_archivo.nombre_archivo.trim().is_empty() {
            return Err(AppError::BadRequest("El nombre del archivo es obligatorio".to_string()));
        }
        if nuevo_archivo.ruta_archivo.trim().is_empty() {
            return Err(AppError::BadRequest("La ruta del archivo es obligatoria".to_string()));
        }
        if nuevo_archivo.tipo_archivo.trim().is_empty() {
            return Err(AppError::BadRequest("El tipo de archivo es obligatorio".to_string()));
        }

        let ahora = Utc::now();
        let archivo = modulo_archivo::ActiveModel {
            modulo_id: Set(nuevo_archivo.modulo_id),
            nombre_archivo: Set(nuevo_archivo.nombre_archivo),
            ruta_archivo: Set(nuevo_archivo.ruta_archivo),
            tipo_archivo: Set(nuevo_archivo.tipo_archivo),
            tamano: Set(nuevo_archivo.tamano),
            descripcion: Set(nuevo_archivo.descripcion),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let archivo_creado = archivo.insert(&self.db).await?;
        Ok(archivo_creado)
    }

    pub async fn obtener_archivos_por_modulo(
        &self,
        modulo_id: i32,
    ) -> Result<Vec<ModuloArchivoModel>, DbErr> {
        ModuloArchivo::find()
            .filter(modulo_archivo::Column::ModuloId.eq(modulo_id))
            .all(&self.db)
            .await
    }

    pub async fn obtener_archivo_por_id(
        &self,
        id: i32,
    ) -> Result<Option<ModuloArchivoModel>, DbErr> {
        ModuloArchivo::find_by_id(id).one(&self.db).await
    }

    pub async fn actualizar_archivo(
        &self,
        id: i32,
        datos_actualizados: ActualizarModuloArchivo,
    ) -> Result<ModuloArchivoModel, AppError> {
        let archivo = ModuloArchivo::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Archivo no encontrado".to_string()))?;

        let mut archivo: modulo_archivo::ActiveModel = archivo.into();
        let ahora = Utc::now();

        if let Some(nombre) = datos_actualizados.nombre_archivo {
            if nombre.trim().is_empty() {
                return Err(AppError::BadRequest("El nombre no puede estar vacío".to_string()));
            }
            archivo.nombre_archivo = Set(nombre);
        }

        if let Some(ruta) = datos_actualizados.ruta_archivo {
            if ruta.trim().is_empty() {
                return Err(AppError::BadRequest("La ruta no puede estar vacía".to_string()));
            }
            archivo.ruta_archivo = Set(ruta);
        }

        if let Some(tipo) = datos_actualizados.tipo_archivo {
            if tipo.trim().is_empty() {
                return Err(AppError::BadRequest("El tipo no puede estar vacío".to_string()));
            }
            archivo.tipo_archivo = Set(tipo);
        }

        if let Some(tamano) = datos_actualizados.tamano {
            archivo.tamano = Set(tamano);
        }

        if let Some(descripcion) = datos_actualizados.descripcion {
            archivo.descripcion = Set(Some(descripcion));
        }

        archivo.updated_at = Set(Some(ahora));
        let archivo_actualizado = archivo.update(&self.db).await?;

        Ok(archivo_actualizado)
    }

    pub async fn eliminar_archivo(&self, id: i32) -> Result<(), AppError> {
        let archivo = ModuloArchivo::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Archivo no encontrado".to_string()))?;

        archivo.delete(&self.db).await?;
        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<ModuloArchivoModel> for ModuloArchivoService {
    async fn get_all(&self) -> Result<Vec<ModuloArchivoModel>, AppError> {
        ModuloArchivo::find()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<ModuloArchivoModel>, AppError> {
        self.obtener_archivo_por_id(id).await.map_err(Into::into)
    }

    async fn create(&self, data: ModuloArchivoModel) -> Result<ModuloArchivoModel, AppError> {
        self.crear_archivo(NuevoModuloArchivo {
            modulo_id: data.modulo_id,
            nombre_archivo: data.nombre_archivo,
            ruta_archivo: data.ruta_archivo,
            tipo_archivo: data.tipo_archivo,
            tamano: data.tamano,
            descripcion: data.descripcion,
        })
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: ModuloArchivoModel,
    ) -> Result<ModuloArchivoModel, AppError> {
        self.actualizar_archivo(
            id,
            ActualizarModuloArchivo {
                nombre_archivo: Some(data.nombre_archivo),
                ruta_archivo: Some(data.ruta_archivo),
                tipo_archivo: Some(data.tipo_archivo),
                tamano: Some(data.tamano),
                descripcion: data.descripcion,
            },
        )
        .await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_archivo(id).await
    }
}
