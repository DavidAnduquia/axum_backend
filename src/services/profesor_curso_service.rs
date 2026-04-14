use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use chrono::Utc;
use once_cell::sync::OnceCell;
use std::sync::Arc;

use crate::models::profesor_curso::{self, Entity as ProfesorCurso, Model as ProfesorCursoModel};
use crate::utils::errors::AppError;

static PROFESOR_CURSO_SERVICE: OnceCell<Arc<ProfesorCursoService>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct ProfesorCursoService {
    db: DatabaseConnection,
}

impl ProfesorCursoService {
    /// Obtiene la instancia global del servicio, inicializándola si es necesario
    pub fn global(conn: &DatabaseConnection) -> &'static Arc<Self> {
        PROFESOR_CURSO_SERVICE.get_or_init(|| {
            Arc::new(Self { db: conn.clone() })
        })
    }

    // Obtener todas las asignaciones de profesores a cursos
    pub async fn obtener_profesores_cursos(&self) -> Result<Vec<ProfesorCursoModel>, AppError> {
        let asignaciones = ProfesorCurso::find()
            .all(&self.db)
            .await?;

        Ok(asignaciones)
    }

    // Asignar un profesor a un curso
    pub async fn asignar_profesor_curso(
        &self,
        profesor_id: i64,
        curso_id: i32,
        estado: Option<String>,
    ) -> Result<ProfesorCursoModel, AppError> {
        // Verificar si ya existe una asignación para este profesor y curso
        let asignacion_existente = ProfesorCurso::find()
            .filter(profesor_curso::Column::ProfesorId.eq(profesor_id))
            .filter(profesor_curso::Column::CursoId.eq(curso_id))
            .one(&self.db)
            .await?;

        if asignacion_existente.is_some() {
            return Err(AppError::Conflict(
                "Ya existe una asignación para este profesor en el curso especificado".to_string(),
            ));
        }

        let nueva_asignacion = profesor_curso::ActiveModel {
            profesor_id: Set(profesor_id),
            curso_id: Set(curso_id),
            fecha_asignacion: Set(Utc::now()),
            estado: Set(estado.unwrap_or_else(|| "activo".to_string())),
            created_at: Set(Some(Utc::now())),
            updated_at: Set(Some(Utc::now())),
            ..Default::default()
        };

        let asignacion = nueva_asignacion.insert(&self.db).await?;
        Ok(asignacion)
    }

    // Editar una asignación de profesor a curso
    pub async fn editar_asignacion_profesor_curso(
        &self,
        id: i32,
        profesor_id: Option<i64>,
        curso_id: Option<i32>,
        estado: Option<String>,
    ) -> Result<ProfesorCursoModel, AppError> {
        let asignacion = ProfesorCurso::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Asignación no encontrada".to_string()))?;

        let mut asignacion: profesor_curso::ActiveModel = asignacion.into();

        if let Some(prof_id) = profesor_id {
            // Verificar si ya existe otra asignación con el mismo profesor y curso
            if let Some(curso) = curso_id {
                let existe_asignacion = ProfesorCurso::find()
                    .filter(profesor_curso::Column::ProfesorId.eq(prof_id))
                    .filter(profesor_curso::Column::CursoId.eq(curso))
                    .filter(profesor_curso::Column::Id.ne(id))
                    .one(&self.db)
                    .await?;

                if existe_asignacion.is_some() {
                    return Err(AppError::Conflict(
                        "Ya existe una asignación para este profesor en el curso especificado"
                            .to_string(),
                    ));
                }
            }
            asignacion.profesor_id = Set(prof_id);
        }

        if let Some(curso) = curso_id {
            // Si no se proporcionó profesor_id, usamos el existente
            let prof_id = profesor_id.unwrap_or_else(|| asignacion.profesor_id.take().unwrap());
            
            // Verificar si ya existe otra asignación con el mismo profesor y curso
            let existe_asignacion = ProfesorCurso::find()
                .filter(profesor_curso::Column::ProfesorId.eq(prof_id))
                .filter(profesor_curso::Column::CursoId.eq(curso))
                .filter(profesor_curso::Column::Id.ne(id))
                .one(&self.db)
                .await?;

            if existe_asignacion.is_some() {
                return Err(AppError::Conflict(
                    "Ya existe una asignación para este profesor en el curso especificado"
                        .to_string(),
                ));
            }
            
            asignacion.curso_id = Set(curso);
        }

        if let Some(estado_valor) = estado {
            asignacion.estado = Set(estado_valor);
        }

        asignacion.updated_at = Set(Some(Utc::now()));
        let asignacion_actualizada = asignacion.update(&self.db).await?;

        Ok(asignacion_actualizada)
    }

    // Eliminar una asignación de profesor a curso
    pub async fn eliminar_asignacion_profesor_curso(
        &self,
        id: i32,
    ) -> Result<(), AppError> {
        let asignacion = ProfesorCurso::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Asignación no encontrada".to_string()))?;

        let _ = asignacion.delete(&self.db).await?;

        Ok(())
    }

    // Obtener una asignación por ID
    pub async fn obtener_asignacion_por_id(
        &self,
        id: i32,
    ) -> Result<Option<ProfesorCursoModel>, DbErr> {
        ProfesorCurso::find_by_id(id).one(&self.db).await
    }
}

#[async_trait]
impl crate::traits::service::CrudService<ProfesorCursoModel> for ProfesorCursoService {
    async fn get_all(&self) -> Result<Vec<ProfesorCursoModel>, AppError> {
        self.obtener_profesores_cursos().await
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<ProfesorCursoModel>, AppError> {
        self.obtener_asignacion_por_id(id).await.map_err(Into::into)
    }

    async fn create(&self, data: ProfesorCursoModel) -> Result<ProfesorCursoModel, AppError> {
        self.asignar_profesor_curso(data.profesor_id, data.curso_id, Some(data.estado)).await
    }

    async fn update(
        &self,
        id: i32,
        data: ProfesorCursoModel,
    ) -> Result<ProfesorCursoModel, AppError> {
        self.editar_asignacion_profesor_curso(id, Some(data.profesor_id), Some(data.curso_id), Some(data.estado))
            .await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_asignacion_profesor_curso(id).await
    }
}
