pub mod storable;

pub use crate::storage::storable::Storable;

use std::sync::Arc;

use anyhow::anyhow;
use once_cell::sync::OnceCell;
use rusqlite::{params, Connection};
use std::sync::Mutex;

#[derive(Debug)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

static STORAGE: OnceCell<Storage> = OnceCell::new();

impl Storage {
    pub fn init() {
        let conn = Connection::open("./storage.db3").unwrap();

        STORAGE
            .set(Storage {
                conn: Arc::new(Mutex::new(conn)),
            })
            .unwrap();
    }

    pub fn store<T>(object: &T) -> Result<i64, Box<dyn std::error::Error>>
    where
        T: Storable,
    {
        let storage = STORAGE.get().expect("Storage not initialized");
        let table_name = T::get_name();
        println!("Here is the table_name : {table_name}");

        if let Err(e) = Self::create_if_not_exist(&table_name) {
            return Err(anyhow!("Something went wrong : {} ", e.to_string()).into());
        }
        {
            let conn = storage.conn.lock().unwrap();
            let serialized_object = Storable::serialize(object);
            match object.get_store_id() {
                Some(id) => {
                    // Update existing data
                    let query = format!(
                        // language=SQL
                        r#"
                            UPDATE {table_name} SET data = (?1) WHERE id = ?2 
                        "#
                    );
                    if let Err(e) = conn.execute(&query, params![serialized_object, id]) {
                        return Err(anyhow!(
                            "Error unable to write to storage : {}",
                            e.to_string()
                        )
                        .into());
                    }
                    Ok(id)
                }
                None => {
                    // Insert the data
                    // language=SQL
                    let query = format!(r#"INSERT INTO {table_name} (data) VALUES (?1) "#);
                    if let Err(e) = conn.execute(&query, params![serialized_object]) {
                        return Err(anyhow!(
                            "Error unable to write to storage : {}",
                            e.to_string()
                        )
                        .into());
                    }
                    let id = conn.last_insert_rowid();
                    Ok(id)
                }
            }
        }
    }
    pub fn get<T>(id: i64) -> Result<Option<T>, Box<dyn std::error::Error>>
    where
        T: Storable,
    {
        let storage = STORAGE.get().expect("Storage not initialized");
        let table_name = T::get_name();

        let query = format!(r#"SELECT data FROM {table_name} WHERE id ?1"#);
        let data = storage
            .conn
            .lock()
            .unwrap()
            .query_row(&query, [id], |row| Ok(row.get(0).unwrap()))
            .unwrap();

        let storable = T::deserialize(id, &data).unwrap();

        Ok(Some(*storable))
    }
    pub fn get_all<T>() -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        T: Storable,
    {
        let storage = STORAGE.get().expect("Storage not initialized");
        let table_name = T::get_name();
        let query =
            format!(r#"SELECT name FROM sqlite_master WHERE type='table' AND name='{table_name}'"#);

        let table_query_result: Result<String, _> =
            storage
                .conn
                .lock()
                .unwrap()
                .query_row(&query, [], |row| Ok(row.get(0).unwrap()));

        if let Err(e) = table_query_result {
            match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    return Ok(vec![]);
                }
                _ => {
                    return Err(anyhow!("Error reading the database : {e}").into());
                }
            }
        }

        let query = format!("SELECT id, data FROM {table_name}");

        let conn = storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))?;

        let mut objects: Vec<T> = vec![];
        for row in rows {
            let (id, data) = row.unwrap();
            objects.push(*T::deserialize(id, &data).unwrap());
        }

        Ok(objects)
    }
    fn create_if_not_exist(table: &str) -> Result<(), Box<dyn std::error::Error>> {
        let storage = STORAGE.get().expect("Storage not initialized");

        let query = format!(
            r#"
                -- language=sql
                CREATE TABLE IF NOT EXISTS {table} (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    data BLOB,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE TRIGGER IF NOT EXISTS notes_modified_at
                    AFTER UPDATE ON {table}
                    FOR EACH ROW
                    BEGIN
                        UPDATE {table} SET modified_at = datetime('now')
                        WHERE id = NEW.id;
                    END;
            "#
        );

        if let Err(e) = storage.conn.lock().unwrap().execute_batch(&query) {
            return Err(anyhow!(e.to_string()).into());
        }
        Ok(())
    }
}
