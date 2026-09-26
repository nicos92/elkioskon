//! Guards that keep `AGENTS.md` in sync with the code it documents.
//!
//! The doc describes counts, paths and command names that nothing in the
//! compiler cares about, so it silently drifts: it was three features behind
//! when these tests were added. These tests make the drift fail `cargo test`
//! instead of waiting for a human to notice a wrong number.
//!
//! Deliberate scope: the checks assert against the *prose in the doc*, not
//! against a separate machine-readable copy of the counts. A marker comment
//! holding the real numbers would be one more thing to forget to update, and
//! could disagree with the text an agent actually reads.

use crate::domain::entities::PermissionCode;
use crate::infrastructure::database::TABLES;

const AGENTS_MD: &str = include_str!("../../AGENTS.md");
const LIB_RS: &str = include_str!("lib.rs");

const SECTION_DB: &str = "## 7. Modelos de Base de Datos";
const SECTION_COMMANDS: &str = "## 8. API Commands";

/// Returns the body of the `## ` section that starts with `heading`.
///
/// Scoping matters: searching the whole document for a name finds prose that
/// merely mentions it, so a table row could be deleted while a sentence
/// elsewhere still satisfied the check.
fn section(heading: &str) -> &'static str {
    let start = AGENTS_MD
        .find(heading)
        .unwrap_or_else(|| panic!("AGENTS.md no tiene la seccion {heading}"));
    let rest = &AGENTS_MD[start + heading.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    &rest[..end]
}

/// Reads the `tauri::generate_handler![]` list from `lib.rs` source.
///
/// That macro needs literal function paths, so the list cannot come from a
/// constant the way the doc's numbers do. The list is a flat run of bare
/// identifiers with no comments, so reading the source and splitting on commas
/// is enough.
fn registered_commands() -> Vec<String> {
    const OPEN: &str = "generate_handler![";
    let start = LIB_RS.find(OPEN).expect("no generate_handler! in lib.rs") + OPEN.len();
    let body = &LIB_RS[start..];
    let end = body.find(']').expect("unterminated generate_handler! list");
    let commands: Vec<String> = body[..end]
        .split(',')
        .map(|entry| {
            entry
                .trim()
                .rsplit("::")
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .filter(|name| !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .collect();
    assert!(
        !commands.is_empty(),
        "parsed zero commands out of generate_handler!; the test would pass vacuously"
    );
    commands
}

#[test]
fn every_registered_command_is_documented_in_the_command_table() {
    let table = section(SECTION_COMMANDS);
    let commands = registered_commands();
    let missing: Vec<&str> = commands
        .iter()
        .map(String::as_str)
        .filter(|cmd| !table.contains(&format!("`{cmd}`")))
        .collect();

    assert!(
        missing.is_empty(),
        "la tabla de la seccion 8 de AGENTS.md no lista estos comandos de \
         generate_handler!: {missing:?}. Agregalos ahi."
    );
}

#[test]
fn every_registered_command_row_is_removed_when_the_command_is() {
    let table = section(SECTION_COMMANDS);
    let documented = table
        .lines()
        .filter(|line| line.starts_with("| `"))
        .filter_map(|line| line.split('`').nth(1))
        .filter(|name| name.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .collect::<Vec<_>>();

    let commands = registered_commands();
    let stale: Vec<&&str> = documented
        .iter()
        .filter(|name| !commands.iter().any(|c| c == *name))
        .collect();

    assert!(
        stale.is_empty(),
        "la seccion 8 de AGENTS.md documenta comandos que ya no estan \
         registrados en generate_handler!: {stale:?}. Borralos."
    );
}

#[test]
fn permission_count_in_docs_matches_the_enum() {
    let expected = PermissionCode::all().len();
    let phrase = format!("{expected} permisos");

    assert!(
        AGENTS_MD.contains(&phrase),
        "AGENTS.md deberia decir \"{phrase}\" para reflejar PermissionCode::all()"
    );
}

#[test]
fn table_count_in_docs_matches_the_schema() {
    let expected = TABLES.len();
    let phrase = format!("{expected} tablas");

    assert!(
        AGENTS_MD.contains(&phrase),
        "AGENTS.md deberia decir \"{phrase}\" para reflejar schema::TABLES"
    );
}

#[test]
fn docs_show_the_shared_test_helper_not_the_raw_lock() {
    assert!(
        AGENTS_MD.contains("fresh_test_db()"),
        "AGENTS.md deberia documentar fresh_test_db() como el helper de tests con DB"
    );
}

#[test]
fn docs_do_not_recommend_the_antipattern_that_poisoned_the_test_lock() {
    assert!(
        !AGENTS_MD.contains("TEST_LOCK.lock().unwrap()"),
        "AGENTS.md muestra el patron antipattern: tomar el lock a mano hace que un \
         test que falle envenene el mutex y tumbe los demas"
    );
}

#[test]
fn db_section_names_the_real_database_file() {
    let db_section = section(SECTION_DB);

    assert!(
        db_section.contains("el-kioskon-app.db"),
        "la seccion 7 de AGENTS.md deberia nombrar el archivo real de la base: \
         el-kioskon-app.db"
    );
}

#[test]
fn db_section_does_not_stale_the_bare_app_db_name() {
    let db_section = section(SECTION_DB);

    // "`app.db`" cannot match inside "`el-kioskon-app.db`": the leading
    // backtick anchors it, so this only fires on a genuinely wrong name.
    assert!(
        !db_section.contains("`app.db`"),
        "la seccion 7 de AGENTS.md menciona el nombre viejo `app.db`"
    );
}
