use std::collections::HashMap;

use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::storage::{Storable, Storage};

pub type Document = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub id: String,
    store_id: Option<i64>,
    pub document: Document,
}

#[async_trait]
impl Storable for Note {
    fn get_name() -> String {
        "Note".to_string()
    }
    fn get_store_id(&self) -> Option<i64> {
        self.store_id
    }

    fn deserialize(id: i64, object: &Vec<u8>) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        let saved = postcard::from_bytes(object);
        match saved {
            Ok(saved) => Ok(Box::new(saved)),
            Err(e) => Err(anyhow!("Deserialize error : {}", e.to_string()).into()),
        }
    }
    async fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let id = Storage::store(self)?;
        self.store_id = Some(id);
        Ok(())
    }
}

pub struct NoteManager {
    notes: HashMap<String, Note>,
}

impl NoteManager {
    pub fn new() -> Self {
        let vec_notes = Storage::get_all::<Note>().unwrap();
        let mut notes = HashMap::new();
        for note in vec_notes.into_iter() {
            notes.insert(note.id.clone(), note);
        }
        Self { notes }
    }
    pub async fn get_all(&self) -> HashMap<String, Note> {
        self.notes.clone()
    }
    pub async fn get(&self, id: &str) -> Option<Note> {
        self.notes.get(id).map(|note| note.clone())
    }
    pub async fn update(
        &mut self,
        id: &str,
        document: Document,
    ) -> Result<Note, Box<dyn std::error::Error>> {
        let note = self.notes.get_mut(id);
        let note = note.unwrap();
        note.document = document;

        note.save();

        Ok(note.clone())
    }
    pub async fn create(
        &mut self,
        doc: Option<Document>,
    ) -> Result<Note, Box<dyn std::error::Error>> {
        let id = self.notes.len();
        let id = format!("note-{id}");
        let mut note = Note {
            document: if let Some(doc) = doc {
                doc
            } else {
                String::new()
            },
            store_id: None,
            id : id.clone(),
        };

        note.save().await.unwrap();

        self.notes.insert(id, note.clone());

        Ok(note.clone())
    }
}
