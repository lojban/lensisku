use deadpool_postgres::Pool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::models::Permission;

pub struct PermissionCache {
    pool: Pool,
    cache: RwLock<HashMap<String, Vec<Permission>>>,
}

impl PermissionCache {
    // ELI5: We're making a special box (Arc) to hold our permission rules that can be safely shared
    // across different parts of our web server. Like a rulebook that many teachers can look at
    // at the same time to check if students are allowed to do something.
    pub fn new(pool: Pool) -> Arc<Self> {
        Arc::new(Self {
            pool,
            cache: RwLock::new(HashMap::new()),
        })
    }

    pub async fn load_permissions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT r.role::text as role, p.name, p.description
                 FROM role_permissions r 
                 JOIN permissions p ON r.permission_id = p.id",
                &[],
            )
            .await?;

        let mut cache = HashMap::new();
        for row in rows {
            let role: String = row.get("role");
            // Normalize role to lowercase for case-insensitive lookups
            let role_lower = role.to_lowercase();
            let permission = Permission {
                name: row.get("name"),
                description: row.get("description"),
            };
            cache
                .entry(role_lower)
                .or_insert_with(Vec::new)
                .push(permission);
        }

        let mut write_cache = self.cache.write().await;
        *write_cache = cache;

        Ok(())
    }

    pub async fn has_permission(&self, role: String, permission_name: &str) -> bool {
        let cache = self.cache.read().await;
        // Normalize role to lowercase for case-insensitive lookup
        let role_lower = role.to_lowercase();
        cache
            .get(&role_lower)
            .map(|perms| perms.iter().any(|p| p.name == permission_name))
            .unwrap_or(false)
    }

    pub async fn get_permissions_for_role(&self, role: String) -> Vec<Permission> {
        let cache = self.cache.read().await;
        // Normalize role to lowercase for case-insensitive lookup
        let role_lower = role.to_lowercase();
        cache
            .get(&role_lower)
            .map(|v| v.to_vec())
            .unwrap_or_default()
    }
}
