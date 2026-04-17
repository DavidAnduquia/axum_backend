use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contenido_transversal")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub curso_id: Option<i32>,
    pub origen_tipo: String,
    pub origen_id: i32,
    pub profesor_id: Option<i32>,
    pub tipo_contenido: String,
    pub ruta_archivo: Option<String>,
    pub enlace_video: Option<String>,
    pub fecha_subida: Option<DateTime<Utc>>,
    pub privacidad: Option<String>,
    pub descripcion: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoContenido {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub area_conocimiento_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActualizarContenido {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub area_conocimiento_id: Option<i32>,
}