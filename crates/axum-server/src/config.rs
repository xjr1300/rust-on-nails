#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
}

#[allow(clippy::new_without_default)]
impl Config {
    pub fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

        Self { database_url }
    }
}
