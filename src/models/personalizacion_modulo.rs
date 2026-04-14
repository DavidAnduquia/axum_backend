use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "personalizaciones_modulos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub modulo_id: i32,
    pub estilos: Option<Value>,
    pub orden_componentes: Option<Value>,
    pub privacidad_componentes: Option<Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::modulo::Entity",
        from = "Column::ModuloId",
        to = "super::modulo::Column::Id"
    )]
    Modulo,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevaPersonalizacionModulo {
    pub modulo_id: i32,
    pub estilos: Option<Value>,
    pub orden_componentes: Option<Value>,
    pub privacidad_componentes: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActualizarPersonalizacionModulo {
    pub estilos: Option<Value>,
    pub orden_componentes: Option<Value>,
    pub privacidad_componentes: Option<Value>,
}
