use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use tokio::task::JoinHandle;

/// Registro global de jobs corriendo. Cada job se gestiona con un JoinHandle.
static JOBS: OnceLock<Mutex<HashMap<i32, JoinHandle<()>>>> = OnceLock::new();

fn get_jobs_map() -> &'static Mutex<HashMap<i32, JoinHandle<()>>> {
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Función estática para obtener el número de jobs activos
pub fn get_jobs_count() -> usize {
    match get_jobs_map().lock() {
        Ok(jobs) => jobs.len(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to acquire JOBS lock (count)");
            0
        }
    }
}

/// Función estática para limpiar todos los jobs durante shutdown
pub fn cleanup_all_jobs() -> usize {
    match get_jobs_map().lock() {
        Ok(mut jobs) => {
            let count = jobs.len();
            for (job_id, handle) in jobs.drain() {
                handle.abort();
                tracing::info!("🛑 Job {} abortado durante shutdown", job_id);
            }
            if count > 0 {
                tracing::info!("🛑 Todos los jobs ({}) han sido detenidos", count);
            }
            count
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to acquire JOBS lock (cleanup)");
            0
        }
    }
}
