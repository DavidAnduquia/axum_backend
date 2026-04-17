use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "contenidos_plantilla")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub plantilla_curso_id: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub tipo_contenido: String,
    pub orden: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoContenidoPlantilla {
    pub plantilla_curso_id: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub tipo_contenido: String,
    pub orden: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActualizarContenidoPlantilla {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub tipo_contenido: Option<String>,
    pub orden: Option<i32>,
}