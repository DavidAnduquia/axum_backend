use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};

// ==================== ENUMS ====================

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tipo_soporte")]
pub enum TipoSoporte {
    #[sea_orm(string_value = "error")]
    Error,
    #[sea_orm(string_value = "sugerencia")]
    Sugerencia,
    #[sea_orm(string_value = "mejora")]
    Mejora,
    #[sea_orm(string_value = "otro")]
    Otro,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "estado_soporte")]
pub enum EstadoSoporte {
    #[sea_orm(string_value = "recibido")]
    Recibido,
    #[sea_orm(string_value = "en_revision")]
    EnRevision,
    #[sea_orm(string_value = "en_desarrollo")]
    EnDesarrollo,
    #[sea_orm(string_value = "resuelto")]
    Resuelto,
    #[sea_orm(string_value = "rechazado")]
    Rechazado,
    #[sea_orm(string_value = "cerrado")]
    Cerrado,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "prioridad_soporte")]
pub enum PrioridadSoporte {
    #[sea_orm(string_value = "baja")]
    Baja,
    #[sea_orm(string_value = "media")]
    Media,
    #[sea_orm(string_value = "alta")]
    Alta,
    #[sea_orm(string_value = "critica")]
    Critica,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tipo_soporte_seguimiento")]
pub enum TipoSeguimientoSoporte {
    #[sea_orm(string_value = "cambio_estado")]
    CambioEstado,
    #[sea_orm(string_value = "comentario")]
    Comentario,
    #[sea_orm(string_value = "asignacion")]
    Asignacion,
    #[sea_orm(string_value = "resolucion")]
    Resolucion,
    #[sea_orm(string_value = "reapertura")]
    Reapertura,
    #[sea_orm(string_value = "otro")]
    Otro,
}

// ==================== REPORTE ERROR ENTITY ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "soportes", schema_name = "rustdema2")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub usuario_id: Option<i32>,
    pub titulo: String,
    pub descripcion: String,
    pub tipo: TipoSoporte,
    pub prioridad: PrioridadSoporte,
    pub estado: EstadoSoporte,
    pub captura_url: Option<String>,
    pub responsable_id: Option<i32>,
    pub fecha_resolucion: Option<DateTime<Utc>>,
    pub solucion: Option<String>,
    #[sea_orm(column_name = "fecha_creacion")]
    pub created_at: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "fecha_actualizacion")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::UsuarioId",
        to = "super::usuario::Column::Id",
        on_delete = "SetNull"
    )]
    Usuario,
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::ResponsableId",
        to = "super::usuario::Column::Id",
        on_delete = "SetNull"
    )]
    Responsable,
}

impl ActiveModelBehavior for ActiveModel {}

// ==================== DTOs PARA CREAR/ACTUALIZAR ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoSoporteError {
    pub usuario_id: Option<i32>,
    pub titulo: String,
    pub descripcion: String,
    pub tipo: TipoSoporte,
    pub prioridad: Option<PrioridadSoporte>,
    pub captura_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualizarSoporteError {
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub prioridad: Option<PrioridadSoporte>,
    pub estado: Option<EstadoSoporte>,
    pub captura_url: Option<String>,
    pub responsable_id: Option<i32>,
    pub solucion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoSeguimientoSoporte {
    pub soporte_id: i32,
    pub usuario_id: Option<i32>,
    pub tipo: TipoSeguimientoSoporte,
    pub comentario: String,
    pub estado_anterior: Option<EstadoSoporte>,
    pub estado_nuevo: Option<EstadoSoporte>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CambiarEstadoSoporte {
    pub estado_nuevo: EstadoSoporte,
    pub comentario: Option<String>,
    pub usuario_id: Option<i32>,
    pub solucion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsignarResponsableRequest {
    pub responsable_id: i32,
    pub comentario: Option<String>,
}

// ==================== SEGUIMIENTO SOPORTE MODULE ====================

pub mod soporte_seguimiento {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "soportes_seguimiento", schema_name = "rustdema2")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i32,
        pub soporte_id: i32,
        pub usuario_id: Option<i32>,
        pub tipo: TipoSeguimientoSoporte,
        pub comentario: String,
        pub estado_anterior: Option<EstadoSoporte>,
        pub estado_nuevo: Option<EstadoSoporte>,
        #[sea_orm(column_name = "fecha_creacion")]
        pub created_at: Option<DateTime<Utc>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::Entity",
            from = "Column::SoporteId",
            to = "super::Column::Id",
            on_delete = "Cascade"
        )]
        Soporte,
        #[sea_orm(
            belongs_to = "super::super::usuario::Entity",
            from = "Column::UsuarioId",
            to = "super::super::usuario::Column::Id",
            on_delete = "SetNull"
        )]
        Usuario,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
