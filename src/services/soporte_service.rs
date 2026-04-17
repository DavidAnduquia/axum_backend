use axum::extract::FromRef;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::{
    database::DbExecutor,
    models::{
        soporte::{
            self, soporte_seguimiento, ActualizarSoporteError, AsignarResponsableRequest,
            CambiarEstadoSoporte, EstadoSoporte, Model as SoporteErrorModel, NuevoSoporteError,
            NuevoSeguimientoSoporte, PrioridadSoporte, TipoSoporte, TipoSeguimientoSoporte,
        },
        AppState,
    },
    utils::errors::AppError,
};

// Alias para entidades
use soporte::Entity as SoporteError;
use soporte_seguimiento::{Entity as SeguimientoSoporte, Model as SeguimientoModel};

#[derive(Debug, Clone)]
pub struct SoporteService {
    db: DbExecutor,
}

impl FromRef<AppState> for SoporteService {
    fn from_ref(state: &AppState) -> Self {
        let executor = state
            .db
            .clone()
            .expect("Database connection is not available");
        SoporteService::new(executor)
    }
}

impl SoporteService {
    pub fn new(db: DbExecutor) -> Self {
        Self { db }
    }

    // ==================== CRUD BÁSICO REPORTES ====================

    /// Crear un nuevo soportes de error
    pub async fn crear_soporte(
        &self,
        nuevo_soporte: NuevoSoporteError,
    ) -> Result<SoporteErrorModel, AppError> {
        // Validaciones
        if nuevo_soporte.titulo.trim().is_empty() {
            return Err(AppError::BadRequest("El título es obligatorio".into()));
        }
        if nuevo_soporte.descripcion.trim().is_empty() {
            return Err(AppError::BadRequest("La descripción es obligatoria".into()));
        }
        if nuevo_soporte.titulo.len() > 200 {
            return Err(AppError::BadRequest(
                "El título no puede exceder 200 caracteres".into(),
            ));
        }

        let ahora = Utc::now();
        let soportes = soporte::ActiveModel {
            usuario_id: Set(nuevo_soporte.usuario_id),
            titulo: Set(nuevo_soporte.titulo.trim().to_string()),
            descripcion: Set(nuevo_soporte.descripcion.trim().to_string()),
            tipo: Set(nuevo_soporte.tipo),
            prioridad: Set(nuevo_soporte.prioridad.unwrap_or(PrioridadSoporte::Media)),
            estado: Set(EstadoSoporte::Recibido),
            captura_url: Set(nuevo_soporte.captura_url),
            responsable_id: Set(None),
            fecha_resolucion: Set(None),
            solucion: Set(None),
            created_at: Set(Some(ahora)),
            updated_at: Set(Some(ahora)),
            ..Default::default()
        };

        let soportes_creado = soportes.insert(&self.db.connection()).await?;

        // Crear seguimiento inicial automático
        self.crear_seguimiento_interno(
            soportes_creado.id,
            nuevo_soporte.usuario_id,
            TipoSeguimientoSoporte::Otro,
            "Reporte creado y registrado en el sistema".to_string(),
            None,
            Some(EstadoSoporte::Recibido),
        )
        .await?;

        Ok(soportes_creado)
    }

    /// Obtener un soportes por ID
    pub async fn obtener_por_id(&self, id: i32) -> Result<SoporteErrorModel, AppError> {
        let soportes = SoporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        Ok(soportes)
    }

    /// Listar soportes con filtros y paginación
    pub async fn listar_soportes(
        &self,
        estado: Option<EstadoSoporte>,
        tipo: Option<TipoSoporte>,
        prioridad: Option<PrioridadSoporte>,
        usuario_id: Option<i32>,
        responsable_id: Option<i32>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<(Vec<SoporteErrorModel>, u64), AppError> {
        let mut query = SoporteError::find();

        // Aplicar filtros
        if let Some(e) = estado {
            query = query.filter(soporte::Column::Estado.eq(e));
        }
        if let Some(t) = tipo {
            query = query.filter(soporte::Column::Tipo.eq(t));
        }
        if let Some(p) = prioridad {
            query = query.filter(soporte::Column::Prioridad.eq(p));
        }
        if let Some(u) = usuario_id {
            query = query.filter(soporte::Column::UsuarioId.eq(u));
        }
        if let Some(r) = responsable_id {
            query = query.filter(soporte::Column::ResponsableId.eq(r));
        }

        let total = query.clone().count(&self.db.connection()).await?;

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }
        if let Some(offset_val) = offset {
            query = query.offset(offset_val);
        }

        let soportes = query
            .order_by_desc(soporte::Column::CreatedAt)
            .all(&self.db.connection())
            .await?;

        Ok((soportes, total))
    }

    /// Actualizar un soporte existente
    pub async fn actualizar_soporte(
        &self,
        id: i32,
        datos: ActualizarSoporteError,
        usuario_accion: Option<i32>,
    ) -> Result<SoporteErrorModel, AppError> {
        let soportes = SoporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        let estado_anterior = soportes.estado.clone();
        let mut soportes: soporte::ActiveModel = soportes.into();
        let mut cambios = Vec::new();
        let mut cambio_estado = false;

        if let Some(titulo) = datos.titulo {
            if !titulo.trim().is_empty() {
                soportes.titulo = Set(titulo.trim().to_string());
                cambios.push("título actualizado".to_string());
            }
        }

        if let Some(descripcion) = datos.descripcion {
            if !descripcion.trim().is_empty() {
                soportes.descripcion = Set(descripcion.trim().to_string());
                cambios.push("descripción actualizada".to_string());
            }
        }

        if let Some(prioridad) = datos.prioridad {
            soportes.prioridad = Set(prioridad);
            cambios.push("prioridad modificada".to_string());
        }

        if let Some(captura) = datos.captura_url {
            soportes.captura_url = Set(Some(captura));
            cambios.push("captura de pantalla actualizada".to_string());
        }

        if let Some(nuevo_estado) = datos.estado {
            soportes.estado = Set(nuevo_estado.clone());
            cambio_estado = true;

            // Si se resuelve, guardar la solución y fecha
            if nuevo_estado == EstadoSoporte::Resuelto {
                soportes.fecha_resolucion = Set(Some(Utc::now()));
                if let Some(solucion) = datos.solucion {
                    soportes.solucion = Set(Some(solucion));
                }
            }
        }

        if let Some(resp_id) = datos.responsable_id {
            soportes.responsable_id = Set(Some(resp_id));
            cambios.push("responsable asignado".to_string());
        }

        soportes.updated_at = Set(Some(Utc::now()));

        let soportes_actualizado = soportes.update(&self.db.connection()).await?;

        // Crear seguimiento automático
        if cambio_estado {
            self.crear_seguimiento_interno(
                soportes_actualizado.id,
                usuario_accion,
                TipoSeguimientoSoporte::CambioEstado,
                format!("Cambio de estado: {:?} -> {:?}", estado_anterior, soportes_actualizado.estado),
                Some(estado_anterior),
                Some(soportes_actualizado.estado.clone()),
            )
            .await?;
        } else if !cambios.is_empty() {
            self.crear_seguimiento_interno(
                soportes_actualizado.id,
                usuario_accion,
                TipoSeguimientoSoporte::Comentario,
                format!("Actualización: {}", cambios.join(", ")),
                None,
                None,
            )
            .await?;
        }

        Ok(soportes_actualizado)
    }

    /// Eliminar un soporte (con todos sus seguimientos por CASCADE)
    pub async fn eliminar_soporte(&self, id: i32) -> Result<(), AppError> {
        let soportes = SoporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        soportes.delete(&self.db.connection()).await?;
        Ok(())
    }

    // ==================== LÓGICA DE NEGOCIO - ESTADOS ====================

    /// Cambiar estado de un soportes con seguimiento automático
    pub async fn cambiar_estado(
        &self,
        id: i32,
        cambio: CambiarEstadoSoporte,
    ) -> Result<SoporteErrorModel, AppError> {
        let soportes = SoporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        let estado_anterior = soportes.estado.clone();
        let estado_nuevo = cambio.estado_nuevo.clone();

        let mut soportes: soporte::ActiveModel = soportes.into();
        soportes.estado = Set(estado_nuevo.clone());
        soportes.updated_at = Set(Some(Utc::now()));

        // Si se resuelve, guardar fecha y solución
        if estado_nuevo == EstadoSoporte::Resuelto {
            soportes.fecha_resolucion = Set(Some(Utc::now()));
            if let Some(solucion) = &cambio.solucion {
                soportes.solucion = Set(Some(solucion.clone()));
            }
        }

        // Si se reabre, limpiar fecha de resolución
        if estado_nuevo == EstadoSoporte::Recibido && estado_anterior == EstadoSoporte::Resuelto {
            soportes.fecha_resolucion = Set(None);
        }

        let soportes_actualizado = soportes.update(&self.db.connection()).await?;

        // Determinar tipo de seguimiento
        let tipo_seguimiento = match (&estado_anterior, &estado_nuevo) {
            (_, EstadoSoporte::Resuelto) => TipoSeguimientoSoporte::Resolucion,
            (EstadoSoporte::Resuelto, _) => TipoSeguimientoSoporte::Reapertura,
            _ => TipoSeguimientoSoporte::CambioEstado,
        };

        let comentario = cambio
            .comentario
            .unwrap_or_else(|| format!("Estado cambiado a: {:?}", estado_nuevo));

        self.crear_seguimiento_interno(
            soportes_actualizado.id,
            cambio.usuario_id,
            tipo_seguimiento,
            comentario,
            Some(estado_anterior),
            Some(estado_nuevo),
        )
        .await?;

        Ok(soportes_actualizado)
    }

    /// Asignar responsable a un soportes
    pub async fn asignar_responsable(
        &self,
        id: i32,
        asignacion: AsignarResponsableRequest,
        usuario_accion: Option<i32>,
    ) -> Result<SoporteErrorModel, AppError> {
        let soportes = SoporteError::find_by_id(id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        let mut soportes: soporte::ActiveModel = soportes.into();
        soportes.responsable_id = Set(Some(asignacion.responsable_id));
        soportes.updated_at = Set(Some(Utc::now()));

        // Si no tiene estado asignado, cambiar a "en_revision"
        let estado_actual = soportes.estado.clone();
        if estado_actual.as_ref() == &EstadoSoporte::Recibido {
            soportes.estado = Set(EstadoSoporte::EnRevision);
        }

        let soportes_actualizado = soportes.update(&self.db.connection()).await?;

        let comentario = asignacion.comentario.unwrap_or_else(|| {
            format!("Responsable asignado: usuario {}", asignacion.responsable_id)
        });

        self.crear_seguimiento_interno(
            soportes_actualizado.id,
            usuario_accion,
            TipoSeguimientoSoporte::Asignacion,
            comentario,
            None,
            None,
        )
        .await?;

        Ok(soportes_actualizado)
    }

    // ==================== SEGUIMIENTO ====================

    /// Crear seguimiento manual (comentario)
    pub async fn crear_seguimiento(
        &self,
        seguimiento: NuevoSeguimientoSoporte,
    ) -> Result<SeguimientoModel, AppError> {
        if seguimiento.comentario.trim().is_empty() {
            return Err(AppError::BadRequest("El comentario es obligatorio".into()));
        }

        // Verificar que el soportes existe
        let _soportes = SoporteError::find_by_id(seguimiento.soporte_id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        let ahora = Utc::now();
        let nuevo_seguimiento = soporte_seguimiento::ActiveModel {
            soporte_id: Set(seguimiento.soporte_id),
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
        soporte_id: i32,
        usuario_id: Option<i32>,
        tipo: TipoSeguimientoSoporte,
        comentario: String,
        estado_anterior: Option<EstadoSoporte>,
        estado_nuevo: Option<EstadoSoporte>,
    ) -> Result<SeguimientoModel, AppError> {
        let ahora = Utc::now();
        let seguimiento = soporte_seguimiento::ActiveModel {
            soporte_id: Set(soporte_id),
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

    /// Obtener seguimientos de un soportes
    pub async fn obtener_seguimientos(
        &self,
        soporte_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<SeguimientoModel>, AppError> {
        // Verificar que el soportes existe
        let _soportes = SoporteError::find_by_id(soporte_id)
            .one(&self.db.connection())
            .await?
            .ok_or_else(|| AppError::NotFound("Soporte no encontrado".into()))?;

        let mut query = SeguimientoSoporte::find()
            .filter(soporte_seguimiento::Column::SoporteId.eq(soporte_id))
            .order_by_desc(soporte_seguimiento::Column::CreatedAt);

        if let Some(l) = limit {
            query = query.limit(l);
        }

        let seguimientos = query.all(&self.db.connection()).await?;
        Ok(seguimientos)
    }

    // ==================== ESTADÍSTICAS ====================

    /// Obtener estadísticas de soportes
    pub async fn obtener_estadisticas(
        &self,
    ) -> Result<EstadisticasReportes, AppError> {
        let total = SoporteError::find().count(&self.db.connection()).await?;

        let recibidos = SoporteError::find()
            .filter(soporte::Column::Estado.eq(EstadoSoporte::Recibido))
            .count(&self.db.connection())
            .await?;

        let en_revision = SoporteError::find()
            .filter(soporte::Column::Estado.eq(EstadoSoporte::EnRevision))
            .count(&self.db.connection())
            .await?;

        let en_desarrollo = SoporteError::find()
            .filter(soporte::Column::Estado.eq(EstadoSoporte::EnDesarrollo))
            .count(&self.db.connection())
            .await?;

        let resueltos = SoporteError::find()
            .filter(soporte::Column::Estado.eq(EstadoSoporte::Resuelto))
            .count(&self.db.connection())
            .await?;

        let rechazados = SoporteError::find()
            .filter(soporte::Column::Estado.eq(EstadoSoporte::Rechazado))
            .count(&self.db.connection())
            .await?;

        let criticos = SoporteError::find()
            .filter(soporte::Column::Prioridad.eq(PrioridadSoporte::Critica))
            .filter(
                soporte::Column::Estado.is_not_in([
                    EstadoSoporte::Resuelto,
                    EstadoSoporte::Rechazado,
                    EstadoSoporte::Cerrado,
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
