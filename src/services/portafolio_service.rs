use once_cell::sync::OnceCell;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter, Set,
};
use std::sync::Arc;

use crate::{
    models::portafolio::{self, Entity as Portafolio, Model as PortafolioModel},
    utils::errors::AppError,
};

pub use crate::models::portafolio::{ActualizarPortafolio, NuevoPortafolio};

// Singleton compartido optimizado
static PORTAFOLIO_SERVICE: OnceCell<Arc<PortafolioService>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct PortafolioService {
    db: DatabaseConnection,
}

impl PortafolioService {
    /// Obtiene la instancia global del servicio, inicializándola si es necesario
    pub fn global(db: &DatabaseConnection) -> &'static Arc<Self> {
        PORTAFOLIO_SERVICE.get_or_init(|| {
            Arc::new(Self { db: db.clone() })
        })
    }

    // Crear un nuevo portafolio
    pub async fn crear_portafolio(
        &self,
        nuevo_portafolio: NuevoPortafolio,
    ) -> Result<PortafolioModel, AppError> {
        // Validar campos obligatorios
        if nuevo_portafolio.nombre.trim().is_empty() {
            return Err(AppError::BadRequest(
                "El nombre del portafolio es obligatorio".into(),
            ));
        }
        let portafolio = portafolio::ActiveModel {
            curso_id: Set(nuevo_portafolio.curso_id),
            nombre: Set(nuevo_portafolio.nombre),
            descripcion: Set(nuevo_portafolio.descripcion),
            ..Default::default()
        };

        let portafolio_creado = portafolio.insert(&self.db).await?;
        Ok(portafolio_creado)
    }

    // Obtener un portafolio por ID
    pub async fn obtener_portafolio_por_id(
        &self,
        id: i32,
    ) -> Result<Option<PortafolioModel>, DbErr> {
        Portafolio::find_by_id(id).one(&self.db).await
    }

    // Obtener portafolios por curso
    pub async fn obtener_portafolios_por_curso(
        &self,
        curso_id: i32,
    ) -> Result<Vec<PortafolioModel>, DbErr> {
        Portafolio::find()
            .filter(portafolio::Column::CursoId.eq(Some(curso_id)))
            .all(&self.db)
            .await
    }

    // Actualizar un portafolio
    pub async fn actualizar_portafolio(
        &self,
        id: i32,
        datos_actualizados: ActualizarPortafolio,
    ) -> Result<PortafolioModel, AppError> {
        let portafolio = Portafolio::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Portafolio no encontrado".into()))?;

        let mut portafolio: portafolio::ActiveModel = portafolio.into();

        if let Some(nombre) = datos_actualizados.nombre {
            if nombre.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "El nombre del portafolio no puede estar vacío".into(),
                ));
            }
            portafolio.nombre = Set(nombre);
        }

        if let Some(descripcion) = datos_actualizados.descripcion {
            portafolio.descripcion = Set(Some(descripcion));
        }

        let portafolio_actualizado = portafolio.update(&self.db).await?;
        Ok(portafolio_actualizado)
    }

    // Eliminar un portafolio
    pub async fn eliminar_portafolio(&self, id: i32) -> Result<(), AppError> {
        let portafolio = Portafolio::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Portafolio no encontrado".into()))?;

        let _ = portafolio.delete(&self.db).await?;
        Ok(())
    }
}
