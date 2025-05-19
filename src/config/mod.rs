use dotenv::dotenv;
use std::env;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub app_host: IpAddr,
    pub app_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let app_host = env::var("APP_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse()
            .expect("APP_HOST must be a valid IP address");
        let app_port = env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("APP_PORT must be a valid port number");

        Self {
            database_url,
            jwt_secret,
            app_host,
            app_port,
        }
    }

    pub fn server_addr(&self) -> SocketAddr {
        SocketAddr::new(self.app_host, self.app_port)
    }
}
