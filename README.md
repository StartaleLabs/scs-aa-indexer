## Code Architecture for L2 Blockchain Indexer (Rust)
We need to design a modular, scalable, and extensible indexing service that listens to Paymaster events on Ethereum & L2 chains like Sonieum and Minato. This should also allow for future expansion to other blockchain events.

## 1️⃣ High-Level Architecture Overview
# 🔹 Components Overview
Indexer Core - Handles blockchain event streaming and processing.
Storage Layer - Stores indexed data (PostgreSQL, Redis, OpenSearch, or a message queue like Kafka/NATS/RabbitMQ).
API Service - Provides endpoints to query indexed data.
Configuration Layer - Manages environment variables and chain-specific configurations.

## 2️⃣ Core Components & Their Roles
# 🟢 (1) Blockchain Event Listener
Uses Alloy (ethers-rs alternative) to listen for Paymaster contract events.
Can connect to multiple RPC endpoints for redundancy.
Processes logs and filters relevant events.
Supports Sonieum, Minato, and other L2 chains.
Example Flow:

Subscribe to NewBlockHeaders.
Retrieve logs for Paymaster contract events.
Decode logs and send them to the processing queue.

# 🟡 (2) Event Processing Pipeline
Normalizes data from different chains.
Validates and transforms event data.
Pushes to storage layer (Postgres, OpenSearch, Redis, or Message Queue).
Key Features:

Supports batch processing for high throughput.
Handles retries in case of RPC failures.


# 🔵 (3) Storage Layer
The indexed data needs to be stored efficiently. We will support multiple backends:

PostgreSQL - Long-term structured storage.
Redis - Fast lookups and caching.
OpenSearch - Full-text search on event logs.
Kafka/NATS - Streaming for real-time processing.

# 🟣 (4) API Service
Provides an HTTP REST API and GraphQL API to fetch indexed data.
Allows developers to query paymaster event history.
Enables filtering by address, event type, and block range.

# ⚙️ (5) Configuration & Chain Management
Uses .env and config.toml to manage RPC URLs, contracts, and storage settings.
Supports multiple chains with different contract addresses.


[Blockchain Node] --> [Event Listener] --> [Processing Pipeline] --> [Storage]
                    ↳ (Filters & Decodes Logs)     ↳ (Validates & Transforms)   ↳ (Saves to DB, Cache, Search)
