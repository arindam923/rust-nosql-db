use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Date(DateTime<Utc>),
}

impl Document {
    pub fn new() -> Self {
        let now = Utc::now();
        Document {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            data: HashMap::new(),
        }
    }

    pub fn with_data(data: HashMap<String, Value>) -> Self {
        let mut doc = Self::new();
        doc.data = data;
        doc
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.updated_at = Utc::now();
        self.data.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.updated_at = Utc::now();
        self.data.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.updated_at = Utc::now();
        self.data.clear();
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.data.values()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
