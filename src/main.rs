use std::path::Path;

use document::{Document, Value};
use storage::StorageEngine;

mod dbtest;
pub mod document;
pub mod query;
pub mod storage;

pub fn create_test_document(name: &str, age: i64, city: &str) -> Document {
    let mut doc = Document::new();
    doc.insert("name".to_string(), Value::String(name.to_string()));
    doc.insert("age".to_string(), Value::Integer(age));
    doc.insert("city".to_string(), Value::String(city.to_string()));
    doc
}

fn main() {
    let db_path = Path::new("test.bin");
    let mut storage = StorageEngine::new(db_path).unwrap();

    let doc1 = create_test_document("Alice", 30, "New York");
    let doc2 = create_test_document("Bob", 25, "San Francisco");
    let doc3 = create_test_document("Charlie", 35, "New York");

    storage
        .write(
            doc1.id().to_string().as_str(),
            &serde_json::to_vec(&doc1).unwrap(),
        )
        .expect("Failed to write doc1");
    storage
        .write(
            doc2.id().to_string().as_str(),
            &serde_json::to_vec(&doc2).unwrap(),
        )
        .expect("Failed to write doc2");
    storage
        .write(
            doc3.id().to_string().as_str(),
            &serde_json::to_vec(&doc3).unwrap(),
        )
        .expect("Failed to write doc3");
}
