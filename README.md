# NOSQL-DB In Rust

a fun and efficient NoSQL database written in Rust! This project includes a document structure, a query engine, and a optimized storage engine. Whether you're a seasoned Rustacean or just getting started with systems programming, this project is a great way to explore the world of database internals.

#### Features

- Document Structure: Define and store complex data models with ease.
- Query Engine: Execute efficient queries to fetch your data quickly.
- Storage Engine: Durable and high-performance storage for your documents.
- Modular Design: Easily extend and customize the components to fit your needs.

#### Getting Started

##### Prerequisites

Here's a quick example to get you started with db:

```rust

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

```

#### Contributing

Contributions are welcome! If you'd like to contribute, please fork the repository and use a feature branch. Pull requests are warmly welcome.

```bash
    Fork the repository.
    Create your feature branch (git checkout -b feature/AmazingFeature).
    Commit your changes (git commit -m 'Add some AmazingFeature').
    Push to the branch (git push origin feature/AmazingFeature).
    Open a pull request.
```

#### License

Distributed under the MIT License. See LICENSE for more information.
