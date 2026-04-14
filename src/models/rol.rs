use crate::database::migrator::AutoMigrate;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, FromRow)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(unique)]
    pub nombre: String,
    #[sea_orm(column_name = "fecha_creacion")]
    pub fecha_creacion: DateTime<Utc>,
    #[sea_orm(column_name = "fecha_actualizacion")]
    pub fecha_actualizacion: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Implementación del trait genérico
impl AutoMigrate for Entity {
    fn entity_name() -> &'static str {
        "roles"
    }

    fn seed_data() -> Option<Vec<String>> {
        Some(vec![r#"
            INSERT INTO rustdema2.roles (nombre) 
            VALUES ('admin'), ('user'), ('moderator')
            ON CONFLICT (nombre) DO NOTHING
            "#
        .to_string()])
    }
}
