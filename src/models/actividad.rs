use chrono::NaiveTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "actividades")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub curso_id: i32,
    pub profesor_id: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub fecha_inicio: NaiveTime,
    pub fecha_fin: NaiveTime,
    pub tipo_actividad: String,
    pub privacidad: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewActividad {
    pub curso_id: i32,
    pub profesor_id: i32,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub fecha_inicio: NaiveTime,
    pub fecha_fin: NaiveTime,
    pub tipo_actividad: String,
    pub privacidad: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActividad {
    pub curso_id: Option<i32>,
    pub profesor_id: Option<i32>,
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub fecha_inicio: Option<NaiveTime>,
    pub fecha_fin: Option<NaiveTime>,
    pub tipo_actividad: Option<String>,
    pub privacidad: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
