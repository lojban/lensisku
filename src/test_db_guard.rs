//! Guard for database-backed tests: tests that
//! write must only ever reach a local, disposable database.

/// Read `var` and return it, panicking unless every host is loopback and the
/// database name contains "test".
pub fn disposable_database_url(var: &str) -> String {
    let url = std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"));
    if let Err(reason) = check_disposable(&url) {
        panic!("{var} rejected: {reason}");
    }
    url
}

fn check_disposable(url: &str) -> Result<(), String> {
    let config: tokio_postgres::Config = url.parse().map_err(|e| format!("invalid URL: {e}"))?;
    let hosts = config.get_hosts();
    if hosts.is_empty() {
        return Err("no host".into());
    }
    for host in hosts {
        let local = match host {
            tokio_postgres::config::Host::Tcp(h) => {
                matches!(h.as_str(), "localhost" | "127.0.0.1" | "::1")
            }
            tokio_postgres::config::Host::Unix(_) => true,
        };
        if !local {
            return Err("host is not loopback".into());
        }
    }
    match config.get_dbname() {
        Some(name) if name.contains("test") => Ok(()),
        _ => Err("database name must contain \"test\"".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::check_disposable;

    #[test]
    fn accepts_local_test_database() {
        assert!(check_disposable("postgres://u:p@127.0.0.1:55432/rafsi_test").is_ok());
        assert!(check_disposable("postgres://u:p@localhost/wiki_test").is_ok());
    }

    #[test]
    fn rejects_remote_or_non_test_database() {
        assert!(check_disposable("postgres://u:p@db.example.org/rafsi_test").is_err());
        assert!(check_disposable("postgres://u:p@127.0.0.1/lojban_lens").is_err());
        assert!(check_disposable("postgres://u:p@127.0.0.1/").is_err());
    }
}
