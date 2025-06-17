// Database configuration utilities
// src/config/database.rs

use std::env;

/// Database configuration structure
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
}

impl DatabaseConfig {
    /// Create DatabaseConfig from environment variables
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(DatabaseConfig {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            name: env::var("DB_NAME").unwrap_or_else(|_| "polytorus".to_string()),
            user: env::var("DB_USER").unwrap_or_else(|_| "polytorus".to_string()),
            password: env::var("DB_PASSWORD")?,
        })
    }

    /// Generate PostgreSQL connection URL
    pub fn to_connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }

    /// Generate connection URL with SSL mode
    pub fn to_connection_url_with_ssl(&self, ssl_mode: &str) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, ssl_mode
        )
    }
}

/// Redis configuration structure
#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub database: u8,
}

impl RedisConfig {
    /// Create RedisConfig from environment variables
    pub fn from_env() -> Self {
        RedisConfig {
            host: env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .unwrap_or(6379),
            password: env::var("REDIS_PASSWORD").ok(),
            database: env::var("REDIS_DB")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }

    /// Generate Redis connection URL
    pub fn to_connection_url(&self) -> String {
        match &self.password {
            Some(password) => format!(
                "redis://:{}@{}:{}/{}",
                password, self.host, self.port, self.database
            ),
            None => format!("redis://{}:{}/{}", self.host, self.port, self.database),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_database_config_from_env() {
        env::set_var("DB_HOST", "testhost");
        env::set_var("DB_PORT", "5433");
        env::set_var("DB_NAME", "testdb");
        env::set_var("DB_USER", "testuser");
        env::set_var("DB_PASSWORD", "testpass");

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.host, "testhost");
        assert_eq!(config.port, 5433);
        assert_eq!(config.name, "testdb");
        assert_eq!(config.user, "testuser");
        assert_eq!(config.password, "testpass");

        let url = config.to_connection_url();
        assert_eq!(url, "postgresql://testuser:testpass@testhost:5433/testdb");
    }

    #[test]
    fn test_redis_config_from_env() {
        env::set_var("REDIS_HOST", "redishost");
        env::set_var("REDIS_PORT", "6380");
        env::set_var("REDIS_PASSWORD", "redispass");

        let config = RedisConfig::from_env();
        assert_eq!(config.host, "redishost");
        assert_eq!(config.port, 6380);
        assert_eq!(config.password, Some("redispass".to_string()));

        let url = config.to_connection_url();
        assert_eq!(url, "redis://:redispass@redishost:6380/0");
    }
}
