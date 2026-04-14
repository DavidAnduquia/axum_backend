use sea_orm::{Database, DatabaseConnection};

/// Ejecuta todas las migraciones basadas en los modelos
pub async fn run_migrations(
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Running migrations from SeaORM models...");

    // Convertir SQLx pool a SeaORM connection
    let db: DatabaseConnection = Database::connect(database_url)
        .await
        .map_err(|e| sqlx::Error::Configuration(e.to_string().into()))?;

    // Migrar tabla usuarios usando SeaORM
    migrate_usuarios_with_seaorm(&db).await?;

    // Migrar tabla roles usando SeaORM
    migrate_roles_with_seaorm(&db).await?;

    tracing::info!("✅ All migrations completed successfully");
    Ok(())
}

/// Migración para usuarios usando el sistema genérico
async fn migrate_usuarios_with_seaorm(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    use crate::database::migrator::migrate_entity;
    use crate::models::usuario::Entity as Usuario;

    // Pasar la entidad como parámetro
    migrate_entity(db, Usuario).await?;

    Ok(())
}

/// Migración para roles usando el sistema genérico
async fn migrate_roles_with_seaorm(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    use crate::database::migrator::migrate_entity;
    use crate::models::rol::Entity as Rol;

    // Pasar la entidad como parámetro
    migrate_entity(db, Rol).await?;

    Ok(())
}