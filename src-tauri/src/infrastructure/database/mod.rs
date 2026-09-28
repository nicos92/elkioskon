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
// Applying the schema, migrations and seeds to an already-open connection.
// Restoring a backup reuses the live handle instead of swapping the file on
// disk, so it needs to re-run this against the restored data; `init_database`
// is the equivalent for a connection that is not open yet.
pub(crate) use connection::initialize;
// Lets the backup flow and `crate::docs_consistency` read the canonical table
// list; the `schema` module itself is private.
pub(crate) use schema::TABLES;
