# Private Vault

Private Vault is an ongoing Rust project for a small distributed file storage service.
The idea is that a node can receive files over HTTP, split them into content-addressed chunks,
store metadata in SQLite, and replicate chunks and manifests to peer nodes.

The project is still under development and should be treated as an experimental foundation,
not a finished backup system or production-ready storage service.

## Current Features

- HTTP API built with Axum.
- File uploads using `multipart/form-data`.
- File chunking with BLAKE3 hashes.
- Local chunk storage on disk and manifests in SQLite.
- File downloads by reading the manifest and stitching chunks back together.
- Basic peer-to-peer replication between configured nodes.
- Docker Compose setup for three local nodes.

## API Overview

- `GET /health` - health check.
- `POST /files` - upload a file in the multipart field `file`.
- `GET /files/{file_id}` - download a stored file.
- `HEAD /chunks/{hash}` - check whether a chunk exists.
- `PUT /chunks/{hash}` - store a chunk, with hash verification.
- `PUT /manifests/{file_id}` - store a file manifest.

## Run Locally

```bash
cargo run
```

Environment variables:

- `NODE_ID` - node name, defaults to `unknown-node`.
- `PORT` - HTTP port, defaults to `8080`.
- `STORAGE_PATH` - directory for the database and chunks, defaults to `./data`.
- `PEERS` - comma-separated list of peer nodes, for example `http://localhost:8082`.

To start three local nodes:

```bash
docker compose up --build
```

The nodes are exposed on `localhost:8081`, `localhost:8082`, and `localhost:8083`.
