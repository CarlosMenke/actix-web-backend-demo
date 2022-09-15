use std::env;

#[derive(Debug)]
pub struct Application {
    pub redis_uri: String,
    pub domain: String,
    pub database_url: String,
}

impl Default for Application {
    fn default() -> Application {
        Application {
            redis_uri: env::var("REDIS_URL").expect("REDIS_URL must be set in .env."),
            domain: env::var("DOMAIN").expect("DMOAIN must be set in .env."),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env"),
        }
    }
}
