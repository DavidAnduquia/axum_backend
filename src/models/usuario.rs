use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "usuarios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub nombre: String,
    pub documento_nit: String,
    #[sea_orm(unique)]
    pub correo: String,
    #[serde(skip_serializing)]
    pub contrasena: String,
    pub foto_url: Option<String>,
    pub rol_id: i32,
    pub semestre: Option<i32>,
    pub genero: String,
    pub fecha_nacimiento: Date,
    pub estado: bool,
    #[sea_orm(column_name = "fecha_creacion")]
    pub fecha_creacion: DateTime<Utc>,
    #[sea_orm(column_name = "fecha_actualizacion")]
    pub fecha_actualizacion: DateTime<Utc>,
    #[sea_orm(column_name = "fecha_ultima_conexion")]
    pub fecha_ultima_conexion: DateTime<Utc>,
    pub token_primer_ingreso: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "fecha_eliminacion")]
    pub fecha_eliminacion: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "device_token", nullable)]
    pub device_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsuarioConRol {
    #[serde(flatten)]
    pub usuario: Model,
    pub rol: super::rol::Model,
}

/// Información de usuario conectado para el panel de administración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsuarioConectadoInfo {
    pub id: i32,
    pub nombre: String,
    pub correo: String,
    pub documento_nit: String,
    pub rol: String,
    pub fecha_ultima_conexion: DateTime<Utc>,
    pub activo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUsuario {
    pub nombre: String,
    pub documento_nit: String,
    pub correo: String,
    pub contrasena: String,
    pub foto_url: Option<String>,
    pub rol_id: i32,
    pub estado: bool,
    pub semestre: Option<i32>,
    pub genero: String,
    pub fecha_nacimiento: String,
    pub token_primer_ingreso: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUsuario {
    pub nombre: Option<String>,
    pub documento_nit: Option<String>,
    pub correo: Option<String>,
    pub contrasena: Option<String>,
    pub rol_id: Option<i32>,
    pub estado: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::rol::Entity",
        from = "Column::RolId",
        to = "super::rol::Column::Id"
    )]
    Rol,
}

impl Related<super::rol::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rol.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl crate::database::migrator::AutoMigrate for Entity {
    fn entity_name() -> &'static str {
        "usuarios"
    }
}
