use axum::extract::FromRef;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::{
    database::DbExecutor,
    models::{
        reporte_error::{
            self, seguimiento_reporte, ActualizarReporteError, AsignarResponsableRequest,
            CambiarEstadoReporte, EstadoReporte, Model as ReporteErrorModel, NuevoReporteError,
            NuevoSeguimientoReporte, PrioridadReporte, TipoReporte, TipoSeguimiento,
        },
        AppState,
    },
    utils::errors::AppError,
};

// Alias para entidades
use reporte_error::Entity as ReporteError;
use seguimiento_reporte::{Entity as SeguimientoReporte, Model as SeguimientoModel};

#[derive(Debug, Clone)]
pub struct ReporteErrorService {
    db: DbExecutor,
}

impl FromRef<AppState> for ReporteErrorService {
    fn from_ref(state: &AppState) -> Self {
        let executor = state
            .db
            .clone()
            .expect("Database connection is not available");
        ReporteErrorService::new(executor)
    }
}

impl ReporteErrorService {
    pub fn new(db: DbExecutor) -> Self {
        Self { db }
    }

    // ==================== CRUD BÁSICO REPORTES ====================

    /// Crear un nuevo reporte de error
    pub async fn crear_reporte(
        &self,
        nuevo_reporte: NuevoReporteError,
    ) -> Result<ReporteErrorModel, AppError> {
        // Validaciones
        if nuevo_reporte.titulo.trim().is_empty() {
            return Err(AppError::BadRequest("El título es obligatorio".into()));
        }
        if nuevo_reporte.descripcion.trim().is_empty() {
            return Err(AppError::BadRequest("La descripción es obligatoria".into()));
        }
        if nuevo_reporte.titulo.len() > 200 {
            return Err(AppError::BadRequest(
                "El título no puede exceder 200 caracteres".into(),
            ));
        }

        let ahora = Utc::now();
        let reporte = reporte_error::ActiveModel {
            usuario_id: Set(nuevo_reporte.usuario_id),
            titulo: Set(nuevo_reporte.titulo.trim().to_string()),
            descripcion: Set(nuevo_reporte.descripcion.trim().to_string()),
            tipo: Set(nuevo_reporte.tipo),
            prioridad: Set(nuevo_reporte.prioridad.unwrap_or(PrioridadReporte::Media)),
            estado: Set(EstadoReporte::Recibido),
            captura_url: Set(nuevo_reporte.captura_url),
            responsable_id: Set(None),
            fecha_resolucion: Set(None),
            solucion: Set(None),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let reporte_creado = reporte.insert(&self.db.connection()).await?;

        // Crear seguimiento inicial automático
        self.crear_seguimiento_interno(
            reporte_creado.id,
            nuevo_reporte.usuario_id,
            TipoSeguimiento::Otro,
            "Reporte creado y registrado en el sistema".to_string(),
            None,
            Some(EstadoReporte::Recibido),
        )
        .await?;

        Ok(reporte_creado)
    }

    /// Obtener un reporte por ID
    pub async fn obtener_por_id(&self, id: i32) -> Result<ReporteErrorModel, AppError> {
        let reporte = ReporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        Ok(reporte)
    }

    /// Listar reportes con filtros y paginación
    pub async fn listar_reportes(
        &self,
        estado: Option<EstadoReporte>,
        tipo: Option<TipoReporte>,
        prioridad: Option<PrioridadReporte>,
        usuario_id: Option<i32>,
        responsable_id: Option<i32>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<(Vec<ReporteErrorModel>, u64), AppError> {
        let mut query = ReporteError::find();

        // Aplicar filtros
        if let Some(e) = estado {
            query = query.filter(reporte_error::Column::Estado.eq(e));
        }
        if let Some(t) = tipo {
            query = query.filter(reporte_error::Column::Tipo.eq(t));
        }
        if let Some(p) = prioridad {
            query = query.filter(reporte_error::Column::Prioridad.eq(p));
        }
        if let Some(u) = usuario_id {
            query = query.filter(reporte_error::Column::UsuarioId.eq(u));
        }
        if let Some(r) = responsable_id {
            query = query.filter(reporte_error::Column::ResponsableId.eq(r));
        }

        let total = query.clone().count(&self.db.connection()).await?;

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val);
        }

        let reportes = query
            .order_by_desc(reporte_error::Column::CreatedAt)
            .all(&self.db.connection())
            .await?;

        Ok((reportes, total))
    }

    /// Actualizar un reporte
    pub async fn actualizar_reporte(
        &self,
        id: i32,
        datos: ActualizarReporteError,
        usuario_accion: Option<i32>,
    ) -> Result<ReporteErrorModel, AppError> {
        let reporte = ReporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        let estado_anterior = reporte.estado.clone();
        let mut reporte: reporte_error::ActiveModel = reporte.into();
        let mut cambios = Vec::new();
        let mut cambio_estado = false;

        if let Some(titulo) = datos.titulo {
            if !titulo.trim().is_empty() {
                reporte.titulo = Set(titulo.trim().to_string());
                cambios.push("título actualizado".to_string());
            }
        }

        if let Some(descripcion) = datos.descripcion {
            if !descripcion.trim().is_empty() {
                reporte.descripcion = Set(descripcion.trim().to_string());
                cambios.push("descripción actualizada".to_string());
            }
        }

        if let Some(prioridad) = datos.prioridad {
            reporte.prioridad = Set(prioridad);
            cambios.push("prioridad modificada".to_string());
        }

        if let Some(captura) = datos.captura_url {
            reporte.captura_url = Set(Some(captura));
            cambios.push("captura de pantalla actualizada".to_string());
        }

        if let Some(nuevo_estado) = datos.estado {
            reporte.estado = Set(nuevo_estado.clone());
            cambio_estado = true;

            // Si se resuelve, guardar la solución y fecha
            if nuevo_estado == EstadoReporte::Resuelto {
                reporte.fecha_resolucion = Set(Some(Utc::now()));
                if let Some(solucion) = datos.solucion {
                    reporte.solucion = Set(Some(solucion));
                }
            }
        }

        if let Some(resp_id) = datos.responsable_id {
            reporte.responsable_id = Set(Some(resp_id));
            cambios.push("responsable asignado".to_string());
        }

        reporte.updated_at = Set(Some(Utc::now()));

        let reporte_actualizado = reporte.update(&self.db.connection()).await?;

        // Crear seguimiento automático
        if cambio_estado {
            self.crear_seguimiento_interno(
                reporte_actualizado.id,
                usuario_accion,
                TipoSeguimiento::CambioEstado,
                format!("Cambio de estado: {:?} -> {:?}", estado_anterior, reporte_actualizado.estado),
                Some(estado_anterior),
                Some(reporte_actualizado.estado.clone()),
            )
            .await?;
        } else if !cambios.is_empty() {
            self.crear_seguimiento_interno(
                reporte_actualizado.id,
                usuario_accion,
                TipoSeguimiento::Comentario,
                format!("Actualización: {}", cambios.join(", ")),
                None,
                None,
            )
            .await?;
        }

        Ok(reporte_actualizado)
    }

    /// Eliminar un reporte (con todos sus seguimientos por CASCADE)
    pub async fn eliminar_reporte(&self, id: i32) -> Result<(), AppError> {
        let reporte = ReporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        reporte.delete(&self.db.connection()).await?;
        Ok(())
    }

    // ==================== LÓGICA DE NEGOCIO - ESTADOS ====================

    /// Cambiar estado de un reporte con seguimiento automático
    pub async fn cambiar_estado(
        &self,
        id: i32,
        cambio: CambiarEstadoReporte,
    ) -> Result<ReporteErrorModel, AppError> {
        let reporte = ReporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        let estado_anterior = reporte.estado.clone();
        let estado_nuevo = cambio.estado_nuevo.clone();

        let mut reporte: reporte_error::ActiveModel = reporte.into();
        reporte.estado = Set(estado_nuevo.clone());
        reporte.updated_at = Set(Some(Utc::now()));

        // Si se resuelve, guardar fecha y solución
        if estado_nuevo == EstadoReporte::Resuelto {
            reporte.fecha_resolucion = Set(Some(Utc::now()));
            if let Some(solucion) = &cambio.solucion {
                reporte.solucion = Set(Some(solucion.clone()));
            }
        }

        // Si se reabre, limpiar fecha de resolución
        if estado_nuevo == EstadoReporte::Recibido && estado_anterior == EstadoReporte::Resuelto {
            reporte.fecha_resolucion = Set(None);
        }

        let reporte_actualizado = reporte.update(&self.db.connection()).await?;

        // Determinar tipo de seguimiento
        let tipo_seguimiento = match (&estado_anterior, &estado_nuevo) {
            (_, EstadoReporte::Resuelto) => TipoSeguimiento::Resolucion,
            (EstadoReporte::Resuelto, _) => TipoSeguimiento::Reapertura,
            _ => TipoSeguimiento::CambioEstado,
        };

        let comentario = cambio
            .comentario
            .unwrap_or_else(|| format!("Estado cambiado a: {:?}", estado_nuevo));

        self.crear_seguimiento_interno(
            reporte_actualizado.id,
            cambio.usuario_id,
            tipo_seguimiento,
            comentario,
            Some(estado_anterior),
            Some(estado_nuevo),
        )
        .await?;

        Ok(reporte_actualizado)
    }

    /// Asignar responsable a un reporte
    pub async fn asignar_responsable(
        &self,
        id: i32,
        asignacion: AsignarResponsableRequest,
        usuario_accion: Option<i32>,
    ) -> Result<ReporteErrorModel, AppError> {
        let reporte = ReporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        let mut reporte: reporte_error::ActiveModel = reporte.into();
        reporte.responsable_id = Set(Some(asignacion.responsable_id));
        reporte.updated_at = Set(Some(Utc::now()));

        // Si no tiene estado asignado, cambiar a "en_revision"
        let estado_actual = reporte.estado.clone();
        if estado_actual.as_ref() == &EstadoReporte::Recibido {
            reporte.estado = Set(EstadoReporte::EnRevision);
        }

        let reporte_actualizado = reporte.update(&self.db.connection()).await?;

        let comentario = asignacion.comentario.unwrap_or_else(|| {
            format!("Responsable asignado: usuario {}", asignacion.responsable_id)
        });

        self.crear_seguimiento_interno(
            reporte_actualizado.id,
            usuario_accion,
            TipoSeguimiento::Asignacion,
            comentario,
            None,
            None,
        )
        .await?;

        Ok(reporte_actualizado)
    }

    // ==================== SEGUIMIENTO ====================

    /// Crear seguimiento manual (comentario)
    pub async fn crear_seguimiento(
        &self,
        seguimiento: NuevoSeguimientoReporte,
    ) -> Result<SeguimientoModel, AppError> {
        if seguimiento.comentario.trim().is_empty() {
            return Err(AppError::BadRequest("El comentario es obligatorio".into()));
        }

        // Verificar que el reporte existe
        let _reporte = ReporteError::find_by_id(seguimiento.reporte_id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        let ahora = Utc::now();
        let nuevo_seguimiento = seguimiento_reporte::ActiveModel {
            reporte_id: Set(seguimiento.reporte_id),
            usuario_id: Set(seguimiento.usuario_id),
            tipo: Set(seguimiento.tipo),
            comentario: Set(seguimiento.comentario.trim().to_string()),
            estado_anterior: Set(seguimiento.estado_anterior),
            estado_nuevo: Set(seguimiento.estado_nuevo),
            created_at: Set(Some(ahora)),
            ..Default::default()
        };

        let seguimiento_creado = nuevo_seguimiento.insert(&self.db.connection()).await?;
        Ok(seguimiento_creado)
    }

    /// Crear seguimiento interno (usado por otros métodos)
    async fn crear_seguimiento_interno(
        &self,
        reporte_id: i32,
        usuario_id: Option<i32>,
        tipo: TipoSeguimiento,
        comentario: String,
        estado_anterior: Option<EstadoReporte>,
        estado_nuevo: Option<EstadoReporte>,
    ) -> Result<SeguimientoModel, AppError> {
        let ahora = Utc::now();
        let seguimiento = seguimiento_reporte::ActiveModel {
            reporte_id: Set(reporte_id),
            usuario_id: Set(usuario_id),
            tipo: Set(tipo),
            comentario: Set(comentario),
            estado_anterior: Set(estado_anterior),
            estado_nuevo: Set(estado_nuevo),
            created_at: Set(Some(ahora)),
            ..Default::default()
        };

        let seguimiento_creado = seguimiento.insert(&self.db.connection()).await?;
        Ok(seguimiento_creado)
    }

    /// Obtener seguimientos de un reporte
    pub async fn obtener_seguimientos(
        &self,
        reporte_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<SeguimientoModel>, AppError> {
        // Verificar que el reporte existe
        let _reporte = ReporteError::find_by_id(reporte_id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Reporte no encontrado".into()))?;

        let mut query = SeguimientoReporte::find()
            .filter(seguimiento_reporte::Column::ReporteId.eq(reporte_id))
            .order_by_desc(seguimiento_reporte::Column::CreatedAt);

        if let Some(l) = limit {
            query = query.limit(l);
        }

        let seguimientos = query.all(&self.db.connection()).await?;
        Ok(seguimientos)
    }

    // ==================== ESTADÍSTICAS ====================

    /// Obtener estadísticas de reportes
    pub async fn obtener_estadisticas(
        &self,
    ) -> Result<EstadisticasReportes, AppError> {
        let total = ReporteError::find().count(&self.db.connection()).await?;

        let recibidos = ReporteError::find()
            .filter(reporte_error::Column::Estado.eq(EstadoReporte::Recibido))
            .count(&self.db.connection())
            .await?;

        let en_revision = ReporteError::find()
            .filter(reporte_error::Column::Estado.eq(EstadoReporte::EnRevision))
            .count(&self.db.connection())
            .await?;

        let en_desarrollo = ReporteError::find()
            .filter(reporte_error::Column::Estado.eq(EstadoReporte::EnDesarrollo))
            .count(&self.db.connection())
            .await?;

        let resueltos = ReporteError::find()
            .filter(reporte_error::Column::Estado.eq(EstadoReporte::Resuelto))
            .count(&self.db.connection())
            .await?;

        let rechazados = ReporteError::find()
            .filter(reporte_error::Column::Estado.eq(EstadoReporte::Rechazado))
            .count(&self.db.connection())
            .await?;

        let criticos = ReporteError::find()
            .filter(reporte_error::Column::Prioridad.eq(PrioridadReporte::Critica))
            .filter(
                reporte_error::Column::Estado.is_not_in([
                    EstadoReporte::Resuelto,
                    EstadoReporte::Rechazado,
                    EstadoReporte::Cerrado,
                ]),
            )
            .count(&self.db.connection())
            .await?;

        Ok(EstadisticasReportes {
            total,
            recibidos,
            en_revision,
            en_desarrollo,
            resueltos,
            rechazados,
            criticos_pendientes: criticos,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct EstadisticasReportes {
    pub total: u64,
    pub recibidos: u64,
    pub en_revision: u64,
    pub en_desarrollo: u64,
    pub resueltos: u64,
    pub rechazados: u64,
    pub criticos_pendientes: u64,
}

use serde::Serialize;
