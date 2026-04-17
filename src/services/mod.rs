pub mod actividad_service;
pub mod area_conocimiento_service;
pub mod cron_service; // /* Cambio nuevo */ Agregar cron_service al módulo
pub mod curso_service;
pub mod firebaseapp;
pub mod examen_service;
pub mod matricula_service;
pub mod modulo_service;
pub mod rol_service;
pub mod socket_service;
pub mod storage_service; // Servicio de almacenamiento para S3/R2
pub mod tema_service;
pub mod unidad_service;
pub mod usuario_service;
pub mod contenido_unidad_service;

// Servicios con dependencias pendientes (async_trait)
// Descomentar cuando se refactoricen
// pub mod calificacion_service;
// pub mod contenido_plantilla_service;
// pub mod evaluacion_calificacion_service;
// pub mod evento_programado_service;
// pub mod modulo_archivo_service;
pub mod notificacion_service;
pub mod portafolio_service;
pub mod portafolio_contenido_service;
pub mod soporte_service;
pub mod personalizacion_portafolio_service;
// pub mod pregunta_examen_service;
// pub mod profesor_curso_service;

use crate::models::{
    calificacion::{Model as CalificacionModel, Relation as CalificacionRelation},
    contenido_plantilla::{
        Model as ContenidoPlantillaModel, Relation as ContenidoPlantillaRelation,
    },
    evaluacion_calificacion::{
        Model as EvaluacionCalificacionModel, Relation as EvaluacionCalificacionRelation,
    },
    evento_programado::{Model as EventoProgramadoModel, Relation as EventoProgramadoRelation},
    historial_curso_actividad::{
        Model as HistorialCursoActividadModel, Relation as HistorialCursoActividadRelation,
    },
    modulo_archivo::{Model as ModuloArchivoModel, Relation as ModuloArchivoRelation},
    notificacion::{Model as NotificacionModel, Relation as NotificacionRelation},
    portafolio::{Model as PortafolioModel, Relation as PortafolioRelation},
    portafolio_contenido::{
        Model as PortafolioContenidoModel, Relation as PortafolioContenidoRelation,
    },
    pregunta_examen::{Model as PreguntaExamenModel, Relation as PreguntaExamenRelation},
    profesor_curso::{Model as ProfesorCursoModel, Relation as ProfesorCursoRelation},
};

macro_rules! touch_type {
    ($($ty:ty),+ $(,)?) => {
        $(const _: usize = core::mem::size_of::<$ty>();)+
    };
}

touch_type!(
    CalificacionModel,
    CalificacionRelation,
    ContenidoPlantillaModel,
    ContenidoPlantillaRelation,
    EvaluacionCalificacionModel,
    EvaluacionCalificacionRelation,
    EventoProgramadoModel,
    EventoProgramadoRelation,
    HistorialCursoActividadModel,
    HistorialCursoActividadRelation,
    ModuloArchivoModel,
    ModuloArchivoRelation,
    NotificacionModel,
    NotificacionRelation,
    PortafolioModel,
    PortafolioRelation,
    PortafolioContenidoModel,
    PortafolioContenidoRelation,
    PreguntaExamenModel,
    PreguntaExamenRelation,
    ProfesorCursoModel,
    ProfesorCursoRelation,
);