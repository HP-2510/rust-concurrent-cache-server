# Rust-concurrent-cache-server
A concurrent in-memory key-value cache server written in Rust, built with Tokio and DashMap.
Supports multiple TCP clients, TTL expiration, background cleanup, guardrails for resource safety, atomic metrics via stats, and async integration tests.

## Project Goals

This project demonstrates practical systems programming concepts in Rust:

- Asynchronous TCP netowkring
- Concurrent shared state management
- Atomic metrics tracking
- TTL expiration semantics
- Background task coordination
- Defensive resource limits
- Integration testing of async services

## Features

### Core Commands

PING
SET <key> <value...> 
GET <key>
DEL <key>
TTL <key>
EXPIRE <key> <seconds>
STATS
HELP
QUIT

### TTL Semantics

TTL key returns:

- -2 : Key does not exist
- -1 : Key exists but has no expiration
- \>= 0 : Seconds remaining until expiration

Expiration is enforced:

- Lazily on access (GET, TTL)
- Periodically via a background cleanup task (janitor)

## Observability via STATS

The STATS command exposes atomic metrics:

- connections
- commands
- gets
- hits
- misses
- sets
- dels
- ttl
- expire
- expired_removed

Counters are implemented using AtomicU64 for lock-free, thread-safe tracking

## Guardrails (Resource Safety)

To prevent abuse and memory exhaustion:

- MAX_LINE_BYTES : limits maximum request size
- MAX_VALUE_BYTES : limits maximum stored value size

These limits are enfored before parsing or storing data

## Architecture

### Concurrency Model

- Built on Tokio async runtime
- Task-per-connection model
- Each TCP client runs in its own async task
- Shared state managed via Arc

### Shared Store

- Uses DashMap <String, Entry>
- Allows concurrent reads and writes without coarse locking
- Eliminates need for Mutex<Hashmap>

### Expiration Strategy

- Expiry time atored as Instant
- Checked during reads (lazy expiration)
- Periodic cleanup via background janitor task

### Metrics Implementation
- Atomic counters (Atomic U64)
- No global locks required
- Safe under concurrent updates

## Running the Server

### Start server

cargo run

### Server listens on:

127.0.0.1:6379

### Connect via netcat

nc 127.0.0.1 6379

### Demo 

printf "PING\nSET a hello EX 2\nGET a\nTTL a\nSTATS\n" | nc 127.0.0.1 6379
sleep 3
printf "GET a\nTTL a\nSTATS\n" | nc 127.0.0.1 6379

### Testing

Integration tests validate:

- SET/GET roundtrip
- DEL behavior
- TTL expiration
- STATS metrics
- Concurrent client safety

Run tests:

cargo test
