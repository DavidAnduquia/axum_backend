use sqlx::PgPool;
use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDate;

/// Inserta usuarios de prueba en la base de datos
pub async fn seed_users(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🌱 Seeding users...");

    // Crear roles por defecto si no existen
    let roles = vec![
        "Administrador",
        "Profesor",
        "Estudiante",
        "Invitado",
    ];

    for role_name in roles {
        let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM roles WHERE nombre = $1")
            .bind(role_name)
            .fetch_optional(pool)
            .await?;

        if exists.is_none() {
            sqlx::query("INSERT INTO roles (nombre) VALUES ($1)")
                .bind(role_name)
                .execute(pool)
                .await?;

            tracing::info!("✅ Rol creado: {}", role_name);
        } else {
            tracing::info!("ℹ️  Rol ya existe: {}", role_name);
        }
    }

    // Usuarios de prueba con todos los campos requeridos
    let users = vec![
        ("admin@example.com", "admin123", "Administrador", "12345678", "M"),
        ("david.anduquia@aulavirtual.com", "admin123", "David Anduquia", "87654321", "M"),
        ("test@example.com", "admin123", "Usuario Test", "11223344", "O"),
        ("1234567890", "admin123", "Usuario Numérico", "99887766", "F"),
    ];

    for (email, password, name, documento, genero) in users {
        // Verificar si el usuario ya existe
        let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM usuarios WHERE correo = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        if exists.is_none() {
            // Hashear contraseña
            let password_hash = hash(password, DEFAULT_COST)?;

            // Fecha de nacimiento por defecto (30 años)
            let fecha_nacimiento = NaiveDate::from_ymd_opt(1994, 1, 1)
                .ok_or("Invalid date")?;

            // Insertar usuario con todos los campos requeridos
            sqlx::query(
                r#"
                INSERT INTO usuarios (nombre, documento_nit, correo, contrasena, genero, fecha_nacimiento, rol_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(name)
            .bind(documento)
            .bind(email)
            .bind(password_hash)
            .bind(genero)
            .bind(fecha_nacimiento)
            .bind(1) // rol_id por defecto (asumimos que existe un rol con id=1)
            .execute(pool)
            .await?;

            tracing::info!("✅ Usuario creado: {} ({})", email, name);
        } else {
            tracing::info!("ℹ️  Usuario ya existe: {}", email);
        }
    }

    tracing::info!("✅ User seeding completed");
    Ok(())
}
