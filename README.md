## Code Architecture for L2 Blockchain Event Indexer (Rust)
A modular, scalable, and extensible indexing service that listens to any events on Ethereum & L2 chains like Sonieum and Minato. This should also allow for future expansion to other blockchain events. The indexer listens to AA related events as now.

## 1️⃣ High-Level Architecture Overview
# 🔹 Components Overview
Indexer Core - Handles blockchain event streaming and processing.
Storage Layer - Stores indexed data (Redis or Message queue like Kafka/NATS/RabbitMQ).
Configuration Layer - Manages environment variables and chain-specific configurations.

## 2️⃣ Core Components & Their Roles
# 🟢 (1) Blockchain Event Listener
Uses Alloy (ethers-rs alternative) to listen for Paymaster contract events.
Can connect to multiple RPC endpoints for redundancy.
Processes logs and filters relevant events.
Supports Sonieum, Minato, and other L2 chains.
Example Flow:

Subscribe to NewBlockHeaders.
Retrieve logs for Paymaster/EP contract or any other contract  events.
Decode logs and send them to the processing queue.

# 🟡 (2) Event Processing Pipeline
Normalizes data from different chains.
Validates and transforms event data.
Pushes to storage layer (Redis, or Message Queue).
Key Features:

Supports batch processing for high throughput.


# 🔵 (3) Storage Layer
The indexed data needs to be stored efficiently. We will support multiple backends:

Redis - Fast lookups and caching.
Kafka/NATS - Streaming for real-time processing.


# ⚙️ (4) Configuration & Chain Management
Uses .env and config.toml to manage RPC URLs, contracts, events and storage settings.
Supports multiple chains with different contract addresses as well as different event signatures.

## 2️⃣ Architecture

<img width="1158" alt="Screenshot 2025-03-02 at 9 26 01 PM" src="https://github.com/user-attachments/assets/7dda1bd8-0639-4ebb-aabd-78184c1d12b6" />


