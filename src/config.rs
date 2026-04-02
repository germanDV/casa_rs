use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub login_email: String,
    pub password: String,
    pub jwt_secret: String,
}

impl Config {
    fn load() -> Self {
        Self {
            login_email: std::env::var("LOGIN_EMAIL").expect("LOGIN_EMAIL must be set"),
            password: std::env::var("PASSWORD").expect("PASSWORD must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }
}

pub fn init() {
    CONFIG
        .set(Config::load())
        .ok()
        .expect("config already initialized");
}

pub fn get() -> &'static Config {
    CONFIG.get().expect("config not initialized")
}
