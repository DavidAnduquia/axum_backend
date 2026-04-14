use axum::extract::ws::Message;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::{mpsc, RwLock};

use crate::models::socket::{ConnectionInfo, SocketMemoryMetrics};

/// Información de conexión WebSocket con sender para enviar mensajes
#[derive(Clone)]
pub struct SocketConnection {
    pub socket_id: String,
    pub sender: mpsc::UnboundedSender<Message>,
}

/// Estructura para manejar conexiones de WebSocket
/// Equivalente a SocketService en TypeScript
#[derive(Clone)]
pub struct SocketService {
    connections: Arc<RwLock<HashMap<i64, Vec<SocketConnection>>>>, // user_id -> vec of connections
    expired_sessions: Arc<RwLock<HashSet<i64>>>, // user_ids whose sessions have been expired
}

#[allow(dead_code)]
impl SocketService {
    pub fn new() -> Self {
        // Capacidad inicial mínima (crece dinámicamente según necesidad)
        // Para aula virtual pequeña (<20 usuarios), esto es más eficiente
        Self {
            connections: Arc::new(RwLock::new(HashMap::with_capacity(1000))),
            expired_sessions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Marca la sesión de un usuario como expirada y elimina sus conexiones
    /// Esto garantiza que no aparezca en la lista de conectados
    pub async fn expire_user_session(&self, user_id: i64) {
        tracing::info!("🚫 Marcando sesión como expirada para usuario {}", user_id);
        
        // Agregar a la lista de sesiones expiradas
        {
            let mut expired = self.expired_sessions.write().await;
            expired.insert(user_id);
        }
        
        // Cerrar y eliminar todas las conexiones del usuario
        let senders_to_close = {
            let mut connections = self.connections.write().await;
            connections.remove(&user_id)
        };
        
        if let Some(sockets) = senders_to_close {
            for socket in sockets {
                let _ = socket.sender.send(Message::Close(None));
                tracing::info!("🔌 Cerrando socket {} del usuario expirado {}", socket.socket_id, user_id);
            }
        }
    }

    /// Limpia la marca de sesión expirada cuando el usuario se desconecta completamente
    /// Esto permite que el usuario pueda reconectarse más tarde
    pub async fn clear_expired_session(&self, user_id: i64) {
        let mut expired = self.expired_sessions.write().await;
        expired.remove(&user_id);
    }

    /// Registra una nueva conexión de socket para un usuario
    pub async fn add_connection(&self, user_id: i64, socket_id: &str, sender: mpsc::UnboundedSender<Message>) {
        tracing::info!("Usuario {} conectado con socket {}", user_id, socket_id);
        let mut connections = self.connections.write().await;
        connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(SocketConnection {
                socket_id: socket_id.to_string(),
                sender,
            });
    }

    /// Elimina una conexión de socket específica
    pub async fn remove_connection(&self, user_id: i64, socket_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(sockets) = connections.get_mut(&user_id) {
            sockets.retain(|conn| conn.socket_id != socket_id);
            if sockets.is_empty() {
                connections.remove(&user_id);
            }
        }
        tracing::info!(
            "👤 Usuario {} desconectado del socket {}",
            user_id,
            socket_id
        );
    }

    /// Desconecta completamente a un usuario (cierra todos sus sockets)
    /// Usado cuando el usuario hace logout manualmente
    pub async fn disconnect_all_user_sockets(&self, user_id: i64) {
        tracing::info!("🔌 Desconectando todos los sockets del usuario {}", user_id);
        
        // Obtener y eliminar todas las conexiones del usuario
        let sockets_to_close = {
            let mut connections = self.connections.write().await;
            connections.remove(&user_id)
        };
        
        // Cerrar cada socket
        if let Some(sockets) = sockets_to_close {
            for socket in sockets {
                let _ = socket.sender.send(Message::Close(None));
                tracing::info!("🔌 Socket {} del usuario {} cerrado", socket.socket_id, user_id);
            }
        }
    }

    /// Emite una notificación a un usuario específico
    /// Equivalente a emitNotificationToUser en TypeScript
    pub async fn emit_notification_to_user(&self, user_id: i64, notification: Value) {
        let connections = self.connections.read().await;
        if let Some(sockets) = connections.get(&user_id) {
            tracing::info!(
                "📤 Emitiendo notificación al usuario {} ({} conexiones)",
                user_id,
                sockets.len()
            );
            // Enviar el mensaje a través de los sockets
            let message_text = match serde_json::to_string(&notification) {
                Ok(text) => text,
                Err(e) => {
                    tracing::error!("❌ Error serializando notificación: {}", e);
                    return;
                }
            };
            for conn in sockets {
                let msg = Message::Text(message_text.clone().into());
                if let Err(e) = conn.sender.send(msg) {
                    tracing::warn!("⚠️  Error enviando mensaje al socket {}: {}", conn.socket_id, e);
                } else {
                    tracing::debug!("  → Socket {}: mensaje enviado", conn.socket_id);
                }
            }
        } else {
            tracing::warn!("⚠️  Usuario {} no tiene conexiones activas", user_id);
        }
    }

    /// Emite una notificación a múltiples usuarios
    /// Equivalente a emitNotificationToUsers en TypeScript
    pub async fn emit_notification_to_users(&self, user_ids: Vec<i64>, notification: Value) {
        for user_id in user_ids {
            self.emit_notification_to_user(user_id, notification.clone())
                .await;
        }
    }

    /// Emite una notificación broadcast a todos los usuarios conectados
    /// Equivalente a emitNotificationBroadcast en TypeScript
    pub async fn emit_notification_broadcast(&self, notification: Value) {
        let connections = self.connections.read().await;
        let total_users = connections.len();
        let total_connections: usize = connections.values().map(|v| v.len()).sum();
        tracing::info!("📢 Broadcast de notificación a {} usuarios ({} conexiones)", total_users, total_connections);

        // Enviar a todos los usuarios
        let message_text = match serde_json::to_string(&notification) {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("❌ Error serializando notificación: {}", e);
                return;
            }
        };
        
        for (user_id, sockets) in connections.iter() {
            for conn in sockets {
                let msg = Message::Text(message_text.clone().into());
                if let Err(e) = conn.sender.send(msg) {
                    tracing::warn!("⚠️  Error enviando mensaje al socket {}: {}", conn.socket_id, e);
                } else {
                    tracing::debug!("  → Usuario {} Socket {}: mensaje enviado", user_id, conn.socket_id);
                }
            }
        }
    }

    /// Verifica si el servicio de sockets está disponible
    /// Equivalente a isAvailable en TypeScript
    pub async fn is_available(&self) -> bool {
        true // En Rust siempre está disponible si la instancia existe
    }

    /// Obtiene información de conexión
    /// Equivalente a getConnectionInfo en TypeScript
    /// Excluye usuarios cuyas sesiones han sido expiradas
    pub async fn get_connection_info(&self) -> ConnectionInfo {
        let connections = self.connections.read().await;
        let expired = self.expired_sessions.read().await;
        
        // Filtrar usuarios expirados
        let active_users: Vec<i64> = connections
            .keys()
            .filter(|user_id| !expired.contains(*user_id))
            .copied()
            .collect();
        
        let connected_users = active_users.len();
        let rooms: Vec<String> = active_users
            .into_iter()
            .map(|user_id| format!("user_{}", user_id))
            .collect();

        ConnectionInfo {
            connected_users,
            rooms,
        }
    }

    /// Obtiene el número total de conexiones activas
    pub async fn get_total_connections(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().map(|v| v.len()).sum()
    }

    /// /* Cambio nuevo */ Obtiene métricas detalladas para monitoreo de memoria
    pub async fn get_memory_metrics(&self) -> SocketMemoryMetrics {
        let connections = self.connections.read().await;
        let total_users = connections.len();
        let total_connections = connections.values().map(|v| v.len()).sum();
        let total_capacity: usize = connections.values().map(|v| v.capacity()).sum();
        let memory_overhead = total_capacity.saturating_sub(total_connections);

        SocketMemoryMetrics {
            total_users,
            total_connections,
            total_capacity,
            memory_overhead,
            largest_user_connections: connections.values().map(|v| v.len()).max().unwrap_or(0),
        }
    }

    /// /* Cambio nuevo */ Optimiza memoria aplicando shrink_to_fit cuando hay overhead significativo
    pub async fn optimize_memory(&self) -> usize {
        let mut connections = self.connections.write().await;
        let mut optimized_count = 0;

        for (user_id, sockets) in connections.iter_mut() {
            let overhead = sockets.capacity().saturating_sub(sockets.len());
            // Solo optimizar si hay más del 50% de capacidad no utilizada y al menos 10 slots vacíos
            if overhead > sockets.len() && overhead >= 10 {
                sockets.shrink_to_fit();
                optimized_count += 1;
                tracing::debug!(
                    "🔧 Optimizada memoria para usuario {}: {} slots liberados",
                    user_id,
                    overhead
                );
            }
        }

        if optimized_count > 0 {
            tracing::info!(
                "🔧 Optimización de memoria completada: {} usuarios optimizados",
                optimized_count
            );
        }

        optimized_count
    }
}

impl Default for SocketService {
    fn default() -> Self {
        Self::new()
    }
}

/// Instancia singleton global del SocketService
/// Equivalente a socketService = SocketService.getInstance() en TypeScript
pub static SOCKET_SERVICE: OnceLock<SocketService> = OnceLock::new();

/// Función helper para obtener la instancia global
pub fn get_socket_service() -> &'static SocketService {
    SOCKET_SERVICE.get_or_init(|| SocketService::new())
}
