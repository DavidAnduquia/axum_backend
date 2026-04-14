#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionInfo {
    pub connected_users: usize,
    pub rooms: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SocketMemoryMetrics {
    pub total_users: usize,
    pub total_connections: usize,
    pub total_capacity: usize,
    pub memory_overhead: usize,
    pub largest_user_connections: usize,
}
