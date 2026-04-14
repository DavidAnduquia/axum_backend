use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use serde_json::{json, Value as JsonValue};
use chrono::Utc;

use crate::{
    models::pregunta_examen::{self, Entity as PreguntaExamen, Model as PreguntaExamenModel},
    utils::errors::AppError,
};

#[derive(Debug, Clone)]
pub struct PreguntaExamenService {
    db: DatabaseConnection,
}

impl PreguntaExamenService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // Crear una nueva pregunta de examen
    pub async fn crear_pregunta_examen(
        &self,
        examen_id: i32,
        pregunta: String,
        tipo_pregunta: String,
        opciones: Option<Vec<String>>,
        respuesta_correcta: Option<JsonValue>,
        valor_puntos: i32,
        orden: i32,
    ) -> Result<PreguntaExamenModel, AppError> {
        // Convertir el vector de opciones a JSON si está presente
        let opciones_json = opciones.map(|opts| {
            if opts.is_empty() {
                json!(null)
            } else if opts.len() == 1 {
                json!(opts[0])
            } else {
                json!(opts)
            }
        });

        let nueva_pregunta = pregunta_examen::ActiveModel {
            examen_id: Set(examen_id),
            pregunta: Set(pregunta),
            tipo_pregunta: Set(tipo_pregunta),
            opciones: Set(opciones_json),
            respuesta_correcta: Set(respuesta_correcta),
            valor_puntos: Set(valor_puntos),
            orden: Set(orden),
            created_at: Set(Some(Utc::now())),
            updated_at: Set(Some(Utc::now())),
            ..Default::default()
        };

        let pregunta_creada = nueva_pregunta.insert(&self.db).await?;
        Ok(pregunta_creada)
    }

    // Obtener preguntas por ID de examen
    pub async fn obtener_preguntas_por_examen(
        &self,
        examen_id: i32,
    ) -> Result<Vec<PreguntaExamenModel>, AppError> {
        let preguntas = PreguntaExamen::find()
            .filter(pregunta_examen::Column::ExamenId.eq(examen_id))
            .order_by_asc(pregunta_examen::Column::Orden)
            .all(&self.db)
            .await?;

        Ok(preguntas)
    }

    // Obtener una pregunta por su ID
    pub async fn obtener_pregunta_por_id(
        &self,
        id: i32,
    ) -> Result<Option<PreguntaExamenModel>, DbErr> {
        PreguntaExamen::find_by_id(id).one(&self.db).await
    }

    // Actualizar una pregunta existente
    pub async fn actualizar_pregunta(
        &self,
        id: i32,
        pregunta: Option<String>,
        tipo_pregunta: Option<String>,
        opciones: Option<Vec<String>>,
        respuesta_correcta: Option<JsonValue>,
        valor_puntos: Option<i32>,
        orden: Option<i32>,
    ) -> Result<PreguntaExamenModel, AppError> {
        let pregunta_actual = PreguntaExamen::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Pregunta no encontrada".to_string()))?;

        let mut pregunta: pregunta_examen::ActiveModel = pregunta_actual.into();

        if let Some(preg) = pregunta_ {
            pregunta.pregunta = Set(preg);
        }

        if let Some(tipo) = tipo_pregunta {
            pregunta.tipo_pregunta = Set(tipo);
        }

        if let Some(opts) = opciones {
            let opciones_json = if opts.is_empty() {
                json!(null)
            } else if opts.len() == 1 {
                json!(opts[0])
            } else {
                json!(opts)
            };
            pregunta.opciones = Set(Some(opciones_json));
        }

        if let Some(respuesta) = respuesta_correcta {
            pregunta.respuesta_correcta = Set(Some(respuesta));
        }

        if let Some(puntos) = valor_puntos {
            pregunta.valor_puntos = Set(puntos);
        }

        if let Some(orden_valor) = orden {
            pregunta.orden = Set(orden_valor);
        }

        pregunta.updated_at = Set(Some(Utc::now()));
        let pregunta_actualizada = pregunta.update(&self.db).await?;

        Ok(pregunta_actualizada)
    }

    // Eliminar una pregunta
    pub async fn eliminar_pregunta(&self, id: i32) -> Result<(), AppError> {
        let pregunta = PreguntaExamen::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Pregunta no encontrada".to_string()))?;

        let _ = pregunta.delete(&self.db).await?;

        Ok(())
    }
}

#[async_trait]
impl crate::traits::service::CrudService<PreguntaExamenModel> for PreguntaExamenService {
    async fn get_all(&self) -> Result<Vec<PreguntaExamenModel>, AppError> {
        PreguntaExamen::find()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<PreguntaExamenModel>, AppError> {
        self.obtener_pregunta_por_id(id).await.map_err(Into::into)
    }

    async fn create(&self, data: PreguntaExamenModel) -> Result<PreguntaExamenModel, AppError> {
        self.crear_pregunta_examen(
            data.examen_id,
            data.pregunta,
            data.tipo_pregunta,
            data.opciones.and_then(|opts| {
                match opts {
                    JsonValue::String(s) => Some(vec![s]),
                    JsonValue::Array(arr) => arr
                        .into_iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    _ => None,
                }
            }),
            data.respuesta_correcta,
            data.valor_puntos,
            data.orden,
        )
        .await
    }

    async fn update(
        &self,
        id: i32,
        data: PreguntaExamenModel,
    ) -> Result<PreguntaExamenModel, AppError> {
        self.actualizar_pregunta(
            id,
            Some(data.pregunta),
            Some(data.tipo_pregunta),
            data.opciones.and_then(|opts| {
                match opts {
                    JsonValue::String(s) => Some(vec![s]),
                    JsonValue::Array(arr) => arr
                        .into_iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    _ => None,
                }
            }),
            data.respuesta_correcta,
            Some(data.valor_puntos),
            Some(data.orden),
        )
        .await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.eliminar_pregunta(id).await
    }
}
