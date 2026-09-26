mod config;
mod connection;
mod maintenance;
mod migrations;
mod schema;
mod seeds;

pub use config::{get_db_path, BCRYPT_COST};
#[cfg(test)]
pub use connection::{fresh_test_db, reset_test_db};
pub use connection::{init_database, DB};
// Lets `crate::docs_consistency` cross-check the table count documented in
// AGENTS.md; the `schema` module itself is private.
#[cfg(test)]
pub(crate) use schema::TABLES;
