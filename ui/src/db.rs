//! The database connection.

use toasty::Db;
use topcoat::context::{Cx, app_context};

/// The local SQLite database file, created when it is first opened.
const DATABASE_URL: &str = "sqlite:data.db";

fn database_url_from_env(value: Option<String>) -> String {
    value.unwrap_or_else(|| DATABASE_URL.to_owned())
}

/// Opens the database with every model in the application.
///
/// The tables come from migrations. After changing a model, generate a migration with
/// `cargo run -- toasty migration generate` and apply it with
/// `cargo run -- toasty migration apply`.
pub async fn connect() -> toasty::Result<Db> {
    let database_url = database_url_from_env(std::env::var("DATABASE_URL").ok());
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&database_url)
        .await
}

/// Returns the database handle from the router's app context. Cloning it is cheap
/// because it refers to a shared connection pool.
pub fn db(cx: &Cx) -> Db {
    app_context::<Db>(cx).clone()
}

#[cfg(test)]
mod tests {
    use super::database_url_from_env;

    #[test]
    fn uses_configured_database_url() {
        assert_eq!(
            database_url_from_env(Some("sqlite:/data/data.db".to_owned())),
            "sqlite:/data/data.db"
        );
    }

    #[test]
    fn defaults_to_local_database_url() {
        assert_eq!(database_url_from_env(None), "sqlite:data.db");
    }
}
