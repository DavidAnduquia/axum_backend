use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

use crate::{
    models::historial_curso_estudiante::{self, Entity as HistorialCursoEstudiante, Model as HistorialCursoEstudianteModel},
    utils::errors::AppError,
};

pub use crate::models::historial_curso_estudiante::{ActualizarHistorial, NuevoHistorial};

#[derive(Debug, Clone)]
pub struct HistorialCursoEstudianteService {
    db: DatabaseConnection,
}

impl HistorialCursoEstudianteService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn crear_historial(
        &self,
        nuevo_historial: NuevoHistorial,
    ) -> Result<HistorialCursoEstudianteModel, AppError> {
        if nuevo_historial.estado.trim().is_empty() {
            return Err(AppError::BadRequest("El estado es obligatorio".to_string()));
        }

        let ahora = Utc::now();
        let historial = historial_curso_estudiante::ActiveModel {
            curso_id: Set(nuevo_historial.curso_id),
            estudiante_id: Set(nuevo_historial.estudiante_id),
            fecha_inscripcion: Set(ahora),
            estado: Set(nuevo_historial.estado),
            calificacion_final: Set(nuevo_historial.calificacion_final),
            aprobado: Set(nuevo_historial.aprobado),
            fecha_creacion: Set(Some(ahora)),
            fecha_actualizacion: Set(Some(ahora)),
            ..Default::default()
        };

        let historial_creado = historial.insert(&self.db).await?;
        Ok(historial_creado)
    }

    pub async fn obtener_historial_completo(
        &self,
    ) -> Result<Vec<HistorialCursoEstudianteModel>, DbErr> {
        HistorialCursoEstudiante::find()
            .all(&self.db)
            .await
    }

    pub async fn obtener_historial_por_ids(
        &self,
        estudiante_id: i64,
        curso_id: i32,
    ) -> Result<Option<HistorialCursoEstudianteModel>, DbErr> {
        HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::EstudianteId.eq(estudiante_id))
            .filter(historial_curso_estudiante::Column::CursoId.eq(curso_id))
            .one(&self.db)
            .await
    }

    pub async fn actualizar_historial(
        &self,
        estudiante_id: i64,
        curso_id: i32,
        datos_actualizados: ActualizarHistorial,
    ) -> Result<HistorialCursoEstudianteModel, AppError> {
        let historial = HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::EstudianteId.eq(estudiante_id))
            .filter(historial_curso_estudiante::Column::CursoId.eq(curso_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Historial no encontrado".to_string()))?;

        let mut historial: historial_curso_estudiante::ActiveModel = historial.into();
        let ahora = Utc::now();

        if let Some(estado) = datos_actualizados.estado {
            if estado.trim().is_empty() {
                return Err(AppError::BadRequest("El estado no puede estar vacío".to_string()));
            }
            historial.estado = Set(estado);
        }

        if let Some(calificacion) = datos_actualizados.calificacion_final {
            historial.calificacion_final = Set(Some(calificacion));
        }

        if let Some(aprobado) = datos_actualizados.aprobado {
            historial.aprobado = Set(aprobado);
        }

        historial.fecha_actualizacion = Set(Some(ahora));
        let historial_actualizado = historial.update(&self.db).await?;

        Ok(historial_actualizado)
    }

    pub async fn eliminar_historial(
        &self,
        estudiante_id: i64,
        curso_id: i32,
    ) -> Result<(), AppError> {
        let historial = HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::EstudianteId.eq(estudiante_id))
            .filter(historial_curso_estudiante::Column::CursoId.eq(curso_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Historial no encontrado".to_string()))?;

        historial.delete(&self.db).await?;
        Ok(())
    }

    pub async fn obtener_historial_por_curso(
        &self,
        curso_id: i32,
    ) -> Result<Vec<HistorialCursoEstudianteModel>, DbErr> {
        HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::CursoId.eq(curso_id))
            .all(&self.db)
            .await
    }

    pub async fn obtener_historial_por_estudiante(
        &self,
        estudiante_id: i64,
    ) -> Result<Vec<HistorialCursoEstudianteModel>, DbErr> {
        HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::EstudianteId.eq(estudiante_id))
            .all(&self.db)
            .await
    }

    pub async fn verificar_matricula(
        &self,
        estudiante_id: i64,
        curso_id: i32,
    ) -> Result<bool, DbErr> {
        let existe = HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::EstudianteId.eq(estudiante_id))
            .filter(historial_curso_estudiante::Column::CursoId.eq(curso_id))
            .count(&self.db)
            .await?;

        Ok(existe > 0)
    }

    pub async fn obtener_calificaciones_estudiante(
        &self,
        estudiante_id: i64,
    ) -> Result<Vec<HistorialCursoEstudianteModel>, DbErr> {
        HistorialCursoEstudiante::find()
            .filter(historial_curso_estudiante::Column::EstudianteId.eq(estudiante_id))
            .filter(historial_curso_estudiante::Column::CalificacionFinal.is_not_null())
            .all(&self.db)
            .await
    }
}

#[async_trait]
impl crate::traits::service::CrudService<HistorialCursoEstudianteModel> for HistorialCursoEstudianteService {
    async fn get_all(&self) -> Result<Vec<HistorialCursoEstudianteModel>, AppError> {
        self.obtener_historial_completo().await.map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<HistorialCursoEstudianteModel>, AppError> {
        // Adaptación para usar el ID primario en lugar de la combinación estudiante_id + curso_id
        HistorialCursoEstudiante::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn create(&self, data: HistorialCursoEstudianteModel) -> Result<HistorialCursoEstudianteModel, AppError> {
        self.crear_historial(NuevoHistorial {
            curso_id: data.curso_id,
            estudiante_id: data.estudiante_id,
            estado: data.estado,
            calificacion_final: data.calificacion_final,
            aprobado: data.aprobado,
        })
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: HistorialCursoEstudianteModel,
    ) -> Result<HistorialCursoEstudianteModel, AppError> {
        // Adaptación para usar el ID primario
        let historial = HistorialCursoEstudiante::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Historial no encontrado".to_string()))?;

        let mut historial: historial_curso_estudiante::ActiveModel = historial.into();
        let ahora = Utc::now();

        historial.estado = Set(data.estado);
        historial.calificacion_final = Set(data.calificacion_final);
        historial.aprobado = Set(data.aprobado);
        historial.fecha_actualizacion = Set(Some(ahora));

        historial.update(&self.db).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        // Adaptación para usar el ID primario
        let historial = HistorialCursoEstudiante::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Historial no encontrado".to_string()))?;

        historial.delete(&self.db).await?;
        Ok(())
    }
}
