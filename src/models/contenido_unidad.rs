use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(schema_name = "rustdema2", table_name = "contenidos_unidad")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub unidad_id: i32,
    pub tipo_contenido: String,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub orden: i32,
    pub contenido: Option<String>,
    pub url: Option<String>,
    pub visible: bool,
    pub obligatorio: bool,
    pub puntos: Option<i32>,
    pub fecha_limite: Option<DateTime<Utc>>,
    pub duracion_estimada: Option<i32>,
    pub examen_id: Option<i32>,
    pub entrega_id: Option<i32>,
    pub fecha_creacion: Option<DateTime<Utc>>,
    pub fecha_actualizacion: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Unidad,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Unidad => Entity::belongs_to(super::unidad::Entity)
                .from(Column::UnidadId)
                .to(super::unidad::Column::Id)
                .into(),
        }
    }
}

impl Related<super::unidad::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Unidad.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoContenidoUnidad {
    pub unidad_id: i32,
    pub tipo_contenido: String,
    pub titulo: String,
    pub descripcion: Option<String>,
    pub contenido: Option<String>,
    pub url: Option<String>,
    pub orden: i32,
    pub visible: bool,
    pub obligatorio: bool,
    pub puntos: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActualizarContenidoUnidad {
    pub tipo_contenido: Option<String>,
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub contenido: Option<String>,
    pub url: Option<String>,
    pub orden: Option<i32>,
    pub visible: Option<bool>,
    pub obligatorio: Option<bool>,
    pub puntos: Option<i32>,
}
