use rusqlite::params;

use crate::domain::entities::DollarQuote;
use crate::domain::repositories::DollarQuoteRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub const MAX_QUOTES: usize = 4;

pub struct SqliteDollarQuoteRepository;

impl Default for SqliteDollarQuoteRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteDollarQuoteRepository {
    pub fn new() -> Self {
        Self
    }
}

fn row_to_quote(row: &rusqlite::Row) -> rusqlite::Result<DollarQuote> {
    Ok(DollarQuote {
        id: row.get(0)?,
        official_buy: row.get(1)?,
        official_sell: row.get(2)?,
        blue_buy: row.get(3)?,
        blue_sell: row.get(4)?,
        timestamp: row.get(5)?,
    })
}

impl DollarQuoteRepository for SqliteDollarQuoteRepository {
    fn save(&self, quote: &DollarQuote) -> Result<DollarQuote, AppError> {
        let mut conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = conn.transaction()?;

        let count: i64 =
            tx.query_row("SELECT COUNT(*) FROM dollar_quotes", [], |row| row.get(0))?;

        if count >= MAX_QUOTES as i64 {
            tx.execute(
                "DELETE FROM dollar_quotes
                 WHERE id = (
                     SELECT id FROM dollar_quotes
                     ORDER BY timestamp ASC, id ASC
                     LIMIT 1
                 )",
                [],
            )?;
        }

        tx.execute(
            "INSERT INTO dollar_quotes (official_buy, official_sell, blue_buy, blue_sell)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                quote.official_buy,
                quote.official_sell,
                quote.blue_buy,
                quote.blue_sell
            ],
        )?;

        let id = tx.last_insert_rowid();
        let saved = tx.query_row(
            "SELECT id, official_buy, official_sell, blue_buy, blue_sell, timestamp
             FROM dollar_quotes WHERE id = ?1",
            params![id],
            row_to_quote,
        )?;

        tx.commit()?;

        Ok(saved)
    }

    fn find_all(&self) -> Result<Vec<DollarQuote>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, official_buy, official_sell, blue_buy, blue_sell, timestamp
             FROM dollar_quotes
             ORDER BY timestamp DESC, id DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![MAX_QUOTES as i64], row_to_quote)?;

        let mut quotes = Vec::new();
        for row in rows {
            quotes.push(row?);
        }

        Ok(quotes)
    }

    fn delete_by_id(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let affected = conn.execute("DELETE FROM dollar_quotes WHERE id = ?1", params![id])?;

        if affected == 0 {
            return Err(AppError::DollarQuoteNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use std::sync::MutexGuard;

    fn fresh_db() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        reset_test_db().unwrap();
        guard
    }

    fn sample_quote(official_buy: f64) -> DollarQuote {
        DollarQuote::new(
            official_buy,
            official_buy + 40.0,
            official_buy + 200.0,
            official_buy + 240.0,
        )
    }

    #[test]
    fn save_assigns_id_and_timestamp() {
        let _guard = fresh_db();
        let repo = SqliteDollarQuoteRepository::new();

        let saved = repo.save(&sample_quote(1000.0)).unwrap();

        assert!(saved.id > 0);
        assert!(!saved.timestamp.is_empty());
        assert_eq!(saved.official_buy, 1000.0);
        assert_eq!(saved.official_sell, 1040.0);
        assert_eq!(saved.blue_buy, 1200.0);
        assert_eq!(saved.blue_sell, 1240.0);
    }

    #[test]
    fn keeps_at_most_4_quotes_after_5_insertions() {
        let _guard = fresh_db();
        let repo = SqliteDollarQuoteRepository::new();

        for i in 1..=5 {
            repo.save(&sample_quote(i as f64 * 100.0)).unwrap();
        }

        let quotes = repo.find_all().unwrap();
        assert_eq!(quotes.len(), MAX_QUOTES);

        let first = quotes.first().unwrap();
        assert_eq!(first.official_buy, 500.0);
    }

    #[test]
    fn find_all_returns_newest_first() {
        let _guard = fresh_db();
        let repo = SqliteDollarQuoteRepository::new();

        for i in 1..=4 {
            repo.save(&sample_quote(i as f64 * 100.0)).unwrap();
        }

        let quotes = repo.find_all().unwrap();
        let buys: Vec<f64> = quotes.iter().map(|q| q.official_buy).collect();
        assert_eq!(buys, vec![400.0, 300.0, 200.0, 100.0]);
    }

    #[test]
    fn delete_by_id_removes_only_the_requested_quote() {
        let _guard = fresh_db();
        let repo = SqliteDollarQuoteRepository::new();

        repo.save(&sample_quote(100.0)).unwrap();
        repo.save(&sample_quote(200.0)).unwrap();
        let target = repo.save(&sample_quote(300.0)).unwrap();

        repo.delete_by_id(target.id).unwrap();

        let quotes = repo.find_all().unwrap();
        assert_eq!(quotes.len(), 2);
        assert!(quotes.iter().all(|q| q.id != target.id));
    }

    #[test]
    fn delete_by_id_errors_for_unknown_id() {
        let _guard = fresh_db();
        let repo = SqliteDollarQuoteRepository::new();

        assert!(matches!(
            repo.delete_by_id(999).unwrap_err(),
            AppError::DollarQuoteNotFound
        ));
    }

    #[test]
    fn delete_opens_room_for_next_insertion() {
        let _guard = fresh_db();
        let repo = SqliteDollarQuoteRepository::new();

        let first = repo.save(&sample_quote(100.0)).unwrap();
        repo.save(&sample_quote(200.0)).unwrap();
        repo.save(&sample_quote(300.0)).unwrap();
        repo.save(&sample_quote(400.0)).unwrap();
        repo.save(&sample_quote(500.0)).unwrap();

        let quotes = repo.find_all().unwrap();
        assert_eq!(quotes.len(), MAX_QUOTES);
        assert!(quotes.iter().all(|q| q.id != first.id));

        repo.delete_by_id(quotes.first().unwrap().id).unwrap();
        repo.save(&sample_quote(600.0)).unwrap();

        let quotes = repo.find_all().unwrap();
        assert_eq!(quotes.len(), MAX_QUOTES);
        assert_eq!(quotes.first().unwrap().official_buy, 600.0);
    }
}
