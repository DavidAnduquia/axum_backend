use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};

// ==================== ENUMS ====================

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tipo_reporte")]
pub enum TipoReporte {
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
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "estado_reporte")]
pub enum EstadoReporte {
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
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "prioridad_reporte")]
pub enum PrioridadReporte {
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
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tipo_seguimiento")]
pub enum TipoSeguimiento {
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
#[sea_orm(table_name = "reportes_errores", schema_name = "rustdema2")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    pub usuario_id: Option<i32>,
    pub titulo: String,
    pub descripcion: String,
    pub tipo: TipoReporte,
    pub prioridad: PrioridadReporte,
    pub estado: EstadoReporte,
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

// Alias para usar fuera del módulo
pub type ReporteErrorModel = Model;

// ==================== DTOs PARA CREAR/ACTUALIZAR ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoReporteError {
    pub usuario_id: Option<i32>,
    pub titulo: String,
    pub descripcion: String,
    pub tipo: TipoReporte,
    pub prioridad: Option<PrioridadReporte>,
    pub captura_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualizarReporteError {
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub prioridad: Option<PrioridadReporte>,
    pub estado: Option<EstadoReporte>,
    pub captura_url: Option<String>,
    pub responsable_id: Option<i32>,
    pub solucion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoSeguimientoReporte {
    pub reporte_id: i32,
    pub usuario_id: Option<i32>,
    pub tipo: TipoSeguimiento,
    pub comentario: String,
    pub estado_anterior: Option<EstadoReporte>,
    pub estado_nuevo: Option<EstadoReporte>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CambiarEstadoReporte {
    pub estado_nuevo: EstadoReporte,
    pub comentario: Option<String>,
    pub usuario_id: Option<i32>,
    pub solucion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsignarResponsableRequest {
    pub responsable_id: i32,
    pub comentario: Option<String>,
}

// ==================== SEGUIMIENTO REPORTE MODULE ====================

pub mod seguimiento_reporte {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "seguimiento_reportes", schema_name = "rustdema2")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = true)]
        pub id: i32,
        pub reporte_id: i32,
        pub usuario_id: Option<i32>,
        pub tipo: TipoSeguimiento,
        pub comentario: String,
        pub estado_anterior: Option<EstadoReporte>,
        pub estado_nuevo: Option<EstadoReporte>,
        #[sea_orm(column_name = "fecha_creacion")]
        pub created_at: Option<DateTime<Utc>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::Entity",
            from = "Column::ReporteId",
            to = "super::Column::Id",
            on_delete = "Cascade"
        )]
        Reporte,
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
