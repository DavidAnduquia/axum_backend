use axum::extract::FromRef;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use std::sync::Arc;
use tracing::instrument;

use crate::{
    models::{
        rol,
        usuario::{self, Entity as Usuario, Model as UsuarioModel, NewUsuario, UpdateUsuario},
        AppState,
    },
    utils::errors::AppError,
};

// Validación básica de email sin dependencia extra
fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

static USUARIO_SERVICE: OnceCell<Arc<UsuarioService>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct UsuarioService {
    db: DatabaseConnection,
}

impl UsuarioService {
    /// Obtiene la instancia global del servicio, inicializándola si es necesario
    pub fn global(conn: &DatabaseConnection) -> &'static Arc<Self> {
        USUARIO_SERVICE.get_or_init(|| Arc::new(Self { db: conn.clone() }))
    }

    /// Obtiene una conexión del pool de manera eficiente
    async fn get_connection(&self) -> DatabaseConnection {
        self.db.clone()
    }

    // No necesitamos el método begin_transaction separado ya que usamos begin() directamente

    // Iniciar sesión
    #[instrument(skip(self))]
    pub async fn login_usuario(
        &self,
        identificador: &str,
        contrasena: &str,
    ) -> Result<UsuarioModel, AppError> {
        let db = self.get_connection().await;

        // Buscar usuario
        let usuario = Usuario::find()
            .filter(
                usuario::Column::DocumentoNit
                    .eq(identificador)
                    .or(usuario::Column::Correo.eq(identificador)),
            )
            .filter(usuario::Column::Contrasena.eq(contrasena))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("Credenciales inválidas".into()))?;

        // Actualizar última conexión
        let now = Utc::now();
        let mut usuario = usuario.into_active_model();
        usuario.fecha_actualizacion = Set(now);
        usuario.fecha_ultima_conexion = Set(now);
        let usuario = usuario.update(&db).await?;

        Ok(usuario)
    }

    // Cerrar sesión
    #[instrument(skip(self))]
    pub async fn logout_usuario(&self, id: i32) -> Result<UsuarioModel, AppError> {
        let db = self.get_connection().await;

        let usuario = Usuario::find_by_id(id)
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;

        // Actualizar última conexión
        let now = Utc::now();
        let mut usuario = usuario.into_active_model();
        usuario.fecha_actualizacion = Set(now);
        usuario.fecha_ultima_conexion = Set(now);
        let usuario = usuario.update(&db).await?;

        Ok(usuario)
    }

    // Obtener todos los usuarios con su rol
    #[instrument(skip(self))]
    pub async fn obtener_usuarios(&self) -> Result<Vec<usuario::UsuarioConRol>, AppError> {
        let db = self.get_connection().await;

        let usuarios = Usuario::find()
            .find_also_related(rol::Entity)
            .all(&db)
            .await?;

        let usuarios_con_rol = usuarios
            .into_iter()
            .filter_map(|(usuario, rol_opt)| {
                rol_opt.map(|rol| usuario::UsuarioConRol { usuario, rol })
            })
            .collect();

        Ok(usuarios_con_rol)
    }

    /// Obtener usuarios conectados con información completa
    /// Combina los datos del SocketService con la base de datos
    #[instrument(skip(self))]
    pub async fn obtener_usuarios_conectados(
        &self,
        user_ids: Vec<i32>,
    ) -> Result<Vec<usuario::UsuarioConectadoInfo>, AppError> {
        let db = self.get_connection().await;

        // Obtener información de usuarios de la base de datos
        let mut usuarios_info = Vec::new();
        for user_id in user_ids {
            if let Some(usuario) = Usuario::find_by_id(user_id).one(&db).await? {
                // Obtener el rol del usuario
                let rol = usuario::Entity::find_by_id(user_id)
                    .find_also_related(rol::Entity)
                    .one(&db)
                    .await?
                    .and_then(|(_, r)| r);

                usuarios_info.push(usuario::UsuarioConectadoInfo {
                    id: usuario.id,
                    nombre: usuario.nombre.clone(),
                    correo: usuario.correo.clone(),
                    documento_nit: usuario.documento_nit.clone(),
                    rol: rol.map(|r| r.nombre).unwrap_or_else(|| "Sin rol".to_string()),
                    fecha_ultima_conexion: usuario.fecha_ultima_conexion,
                    activo: true,
                });
            }
        }

        Ok(usuarios_info)
    }

    /// Expirar sesión de un usuario (usado por administradores)
    /// Marca la última conexión como ahora para invalidar tokens previos
    #[instrument(skip(self))]
    pub async fn expirar_sesion_usuario(&self, id: i32) -> Result<UsuarioModel, AppError> {
        let db = self.get_connection().await;

        let usuario = Usuario::find_by_id(id)
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;

        // Actualizar la fecha de última conexión a "ahora" para invalidar tokens previos
        // El middleware de auth verificará que el token iat < fecha_ultima_conexion
        let now = Utc::now();
        let mut usuario = usuario.into_active_model();
        usuario.fecha_ultima_conexion = Set(now);
        usuario.fecha_actualizacion = Set(now); // También actualizar fecha_actualizacion para evitar error del trigger
        let usuario = usuario.update(&db).await?;

        tracing::info!("🚫 Sesión expirada para usuario {} por admin", id);

        Ok(usuario)
    }

    // Crear un nuevo usuario
    pub async fn crear_usuario(
        &self,
        nuevo_usuario: NewUsuario,
    ) -> Result<usuario::Model, AppError> {
        let db = self.get_connection().await;
        // Validar campos obligatorios
        if nuevo_usuario.nombre.trim().is_empty() {
            return Err(AppError::BadRequest(
                "El nombre completo es obligatorio".into(),
            ));
        }

        if nuevo_usuario.correo.trim().is_empty() {
            return Err(AppError::BadRequest(
                "El correo electrónico es obligatorio".into(),
            ));
        }

        // Validar formato de correo
        if !is_valid_email(&nuevo_usuario.correo) {
            return Err(AppError::BadRequest(
                "El formato del correo electrónico no es válido".into(),
            ));
        }

        if nuevo_usuario.contrasena.trim().is_empty() {
            return Err(AppError::BadRequest("La contraseña es obligatoria".into()));
        }

        if nuevo_usuario.contrasena.len() < 6 {
            return Err(AppError::BadRequest(
                "La contraseña debe tener al menos 6 caracteres".into(),
            ));
        }

        // Verificar que el correo no esté en uso
        let existe_correo = Usuario::find()
            .filter(usuario::Column::Correo.eq(&nuevo_usuario.correo))
            .one(&db)
            .await?;

        if existe_correo.is_some() {
            return Err(AppError::Conflict(
                "Ya existe un usuario con este correo electrónico".into(),
            ));
        }

        // Verificar que el documento no esté en uso
        if !nuevo_usuario.documento_nit.trim().is_empty() {
            let existe_documento = Usuario::find()
                .filter(usuario::Column::DocumentoNit.eq(&nuevo_usuario.documento_nit))
                .one(&db)
                .await?;

            if existe_documento.is_some() {
                return Err(AppError::Conflict(
                    "Ya existe un usuario con este documento".into(),
                ));
            }
        }

        // Crear el nuevo usuario
        let fecha_nacimiento = nuevo_usuario
            .fecha_nacimiento
            .parse::<chrono::NaiveDate>()
            .map_err(|_| AppError::BadRequest("La fecha de nacimiento es inválida".into()))?;

        let usuario = usuario::ActiveModel {
            nombre: Set(nuevo_usuario.nombre),
            documento_nit: Set(nuevo_usuario.documento_nit),
            correo: Set(nuevo_usuario.correo),
            contrasena: Set(nuevo_usuario.contrasena),
            foto_url: Set(nuevo_usuario.foto_url),
            rol_id: Set(nuevo_usuario.rol_id),
            semestre: Set(nuevo_usuario.semestre),
            genero: Set(nuevo_usuario.genero),
            fecha_nacimiento: Set(fecha_nacimiento),
            estado: Set(nuevo_usuario.estado),
            token_primer_ingreso: Set(nuevo_usuario
                .token_primer_ingreso
                .and_then(|fecha| fecha.parse::<chrono::NaiveDateTime>().ok())
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))),
            fecha_creacion: Set(Utc::now()),
            fecha_actualizacion: Set(Utc::now()),
            fecha_ultima_conexion: Set(Utc::now()),
            ..Default::default()
        };

        let usuario = usuario.insert(&db).await?;
        Ok(usuario)
    }

    // Editar usuario existente
    pub async fn editar_usuario(
        &self,
        id: i32,
        datos_actualizados: UpdateUsuario,
    ) -> Result<usuario::Model, AppError> {
        let db = self.get_connection().await;
        let usuario = Usuario::find_by_id(id)
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;

        let mut usuario: usuario::ActiveModel = usuario.into();

        if let Some(nombre) = datos_actualizados.nombre {
            if nombre.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "El nombre no puede estar vacío".into(),
                ));
            }
            usuario.nombre = Set(nombre);
        }

        if let Some(correo) = datos_actualizados.correo {
            if !is_valid_email(&correo) {
                return Err(AppError::BadRequest(
                    "El formato del correo electrónico no es válido".into(),
                ));
            }
            // Verificar que el nuevo correo no esté en uso por otro usuario
            let existe_correo = Usuario::find()
                .filter(usuario::Column::Correo.eq(&correo))
                .filter(usuario::Column::Id.ne(id))
                .one(&db)
                .await?;

            if existe_correo.is_some() {
                return Err(AppError::Conflict(
                    "Ya existe un usuario con este correo electrónico".into(),
                ));
            }
            usuario.correo = Set(correo);
        }

        if let Some(documento) = datos_actualizados.documento_nit {
            // Verificar que el documento no esté en uso por otro usuario
            if !documento.trim().is_empty() {
                let existe_documento = Usuario::find()
                    .filter(usuario::Column::DocumentoNit.eq(&documento))
                    .filter(usuario::Column::Id.ne(id))
                    .one(&db)
                    .await?;

                if existe_documento.is_some() {
                    return Err(AppError::Conflict(
                        "Ya existe un usuario con este documento".into(),
                    ));
                }
            }
            usuario.documento_nit = Set(documento);
        }

        if let Some(contrasena) = datos_actualizados.contrasena {
            if contrasena.len() < 6 {
                return Err(AppError::BadRequest(
                    "La contraseña debe tener al menos 6 caracteres".into(),
                ));
            }
            usuario.contrasena = Set(contrasena);
        }

        if let Some(rol_id) = datos_actualizados.rol_id {
            usuario.rol_id = Set(rol_id);
        }

        if let Some(estado) = datos_actualizados.estado {
            usuario.estado = Set(estado);
        }

        usuario.fecha_actualizacion = Set(Utc::now());
        let usuario = usuario.update(&db).await?;

        Ok(usuario)
    }

    // Obtener usuario por ID
    pub async fn obtener_usuario_por_id(
        &self,
        id: i32,
    ) -> Result<Option<usuario::Model>, AppError> {
        let db = self.get_connection().await;
        let usuario = Usuario::find_by_id(id).one(&db).await?;
        Ok(usuario)
    }
}

impl FromRef<AppState> for Arc<UsuarioService> {
    fn from_ref(app_state: &AppState) -> Self {
        let db = app_state.db.as_ref().expect("DB no disponible");
        let conn = db.connection();
        UsuarioService::global(&conn).clone()
    }
}
