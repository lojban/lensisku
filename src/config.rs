use crate::{AppError, AppResult};
use deadpool::managed::QueueMode;
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime, Timeouts};

use std::env;
use std::time::Duration;
use tokio_postgres::NoTls;

fn get_env_vars(vars: &[&str]) -> Result<Vec<String>, AppError> {
    let mut values = Vec::new();
    let mut errors = Vec::new();

    for &var in vars {
        match env::var(var) {
            Ok(value) => values.push(value),
            Err(err) => errors.push(format!("{}: {}", var, err)),
        }
    }

    if !errors.is_empty() {
        Err(AppError::Config(errors))
    } else {
        Ok(values)
    }
}

fn parse_pool_size(value: String, var_name: &str) -> Result<usize, String> {
    value.parse().map_err(|e| format!("{}: {}", var_name, e))
}

#[derive(Clone)]
pub struct AppConfig {
    pub db_pools: DatabasePools,
}

pub fn create_app_config() -> AppResult<AppConfig> {
    let db_pools = create_db_pools()?;
    Ok(AppConfig { db_pools })
}

#[derive(Clone)]
pub struct DatabasePools {
    pub app_pool: Pool,
    pub import_pool: Pool,
}

pub fn create_db_pools() -> AppResult<DatabasePools> {
    let mut app_cfg = Config::new();
    let mut import_cfg = Config::new();

    // Collect all required env vars first
    let required_vars = [
        "DB_USER",
        "DB_PASSWORD",
        "DB_NAME",
        "DB_APP_POOL_SIZE",
        "DB_IMPORT_POOL_SIZE",
    ];
    let values = get_env_vars(&required_vars)?;

    // Optional vars with defaults
    let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("DB_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5432);

    // Parse pool sizes
    let mut errors = Vec::new();
    let app_pool_size = match parse_pool_size(values[3].clone(), "DB_APP_POOL_SIZE") {
        Ok(size) => size,
        Err(e) => {
            errors.push(e);
            0 // Temporary value, won't be used if there are errors
        }
    };

    let import_pool_size = match parse_pool_size(values[4].clone(), "DB_IMPORT_POOL_SIZE") {
        Ok(size) => size,
        Err(e) => {
            errors.push(e);
            0 // Temporary value, won't be used if there are errors
        }
    };

    if !errors.is_empty() {
        return Err(AppError::Config(errors));
    }

    // Configure pools with validation and health checks.
    // The app pool serves concurrent HTTP handlers; the assistant may run many parallel
    // `semantic_search` transactions per batched tool call (`ASSISTANT_SEMANTIC_SUBQUERY_CONCURRENCY`).
    // Size `DB_APP_POOL_SIZE` so peak concurrent DB work (assistant + other routes) stays below `max_size`.
    app_cfg.host = Some(host.clone());
    app_cfg.port = Some(port);
    app_cfg.user = Some(values[0].clone());
    app_cfg.password = Some(values[1].clone());
    app_cfg.dbname = Some(values[2].clone());
    app_cfg.pool = Some(PoolConfig {
        max_size: app_pool_size.clamp(5, 100),
        queue_mode: QueueMode::Lifo,
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(30)),
            create: Some(Duration::from_secs(10)),
            recycle: Some(Duration::from_secs(5)),
        },
    });

    import_cfg.host = Some(host);
    import_cfg.port = Some(port);
    import_cfg.user = Some(values[0].clone());
    import_cfg.password = Some(values[1].clone());
    import_cfg.dbname = Some(values[2].clone());
    import_cfg.pool = Some(PoolConfig {
        max_size: import_pool_size.clamp(5, 100),
        queue_mode: QueueMode::Fifo,
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(30)),
            create: Some(Duration::from_secs(10)),
            recycle: Some(Duration::from_secs(5)),
        },
    });

    // Create pools - if this fails, it will return a single error since it's a critical operation
    Ok(DatabasePools {
        app_pool: app_cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AppError::Config(vec![format!("Failed to create app pool: {}", e)]))?,
        import_pool: import_cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AppError::Config(vec![format!("Failed to create import pool: {}", e)]))?,
    })
}
