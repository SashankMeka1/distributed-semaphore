# distributed-semaphore

A custom in-memory store built in Rust with a two-level expiry system and ZSet implementation designed for distributed semaphore use cases. Uses the RESP protocol so it works with any Redis client, but the data model is our own.

## What it is

An in-memory store that mimics Redis's wire protocol over TCP. The core idea: multiple app servers can coordinate shared state through a single store — the classic distributed systems problem Redis solves cheaply, but with per-element expiry that Redis doesn't support.

## Architecture

```
Client (redis-cli / any Redis client)
        │
        │ TCP (port 6379)
        ▼
┌─────────────────────────────────┐
│         TCP Listener            │
├─────────────────────────────────┤
│       RESP Parser               │
├─────────────────────────────────┤
│      Command Dispatcher         │
├─────────────────────────────────┤
│       In-Memory Store           │
├─────────────────────────────────┤
│       TTL / Expiry Worker       │
└─────────────────────────────────┘
```

## Two-Level Expiry System

Unlike standard Redis which only supports key-level TTLs, this implementation has two independent expiry clocks:

```
Store
└── HashMap<String, Entry>              ← Level 1: top-level key expiry
        Entry {
            value: RedisValue,
            expires_at: Option<Instant>
        }

RedisValue {
    String(String)
    List(VecDeque<Element>)
    Hash(HashMap<String, Element>)      ← Level 2: per-element expiry
    Set(HashMap<String, Element>)
    ZSet(HashMap<String, Element>)
}
        Element {
            value: Box<RedisValue>,
            expires_at: Option<Instant>
        }
```

- **Level 1** — the whole key expires, taking all elements with it (cascade)
- **Level 2** — individual elements inside a collection expire independently

The background sweeper runs both levels every second. Collections also sweep lazily on every access.

## Supported Commands

**Server**
- `PING [message]`, `ECHO message`, `FLUSHALL`, `DBSIZE`

**Strings**
- `GET key`
- `SET key value [EX seconds] [PX milliseconds]`
- `DEL key [key ...]`
- `EXISTS key [key ...]`
- `INCR key`
- `EXPIRE key seconds`
- `TTL key`

**Lists**
- `LPUSH key value [value ...]`
- `RPUSH key value [value ...]`
- `LPOP key`
- `RPOP key`
- `LRANGE key start stop`

**Hashes**
- `HSET key field value [field value ...]`
- `HGET key field`
- `HDEL key field [field ...]`
- `HGETALL key`

**Sets**
- `SADD key member [member ...]`
- `SREM key member [member ...]`
- `SMEMBERS key`
- `SISMEMBER key member`

**Sorted Sets (ZSet) — Custom**
- `ZADD key score member [EX seconds]` — add member with score and optional per-member TTL
- `ZREM key member [member ...]`
- `ZCARD key` — count of non-expired members
- `ZSCORE key member`
- `ZRANGE key start stop`
- `ZREMRANGEBYSCORE key min max`

## Distributed Semaphore Pattern

The ZSet per-member expiry makes it well suited for a distributed semaphore. Each member is a client ID, its score is arbitrary, and its TTL is the lock timeout. If a client crashes without releasing, its entry expires automatically on the next acquire.

```
# acquire
ZREMRANGEBYSCORE semaphore 0 <now>   # sweep expired holders
ZCARD semaphore                       # check current holder count
ZADD semaphore 1.0 <client_id> EX 30 # claim a slot, expires in 30s

# release
ZREM semaphore <client_id>
```

Multiple app servers all talking to the same instance coordinate without any additional infrastructure.

## Running

```bash
cargo run
```

Connect with any Redis client:

```bash
redis-cli -p 6379
```

## Running

```bash
cargo run
```

Run the semaphore test:

```bash
./test_semaphore.sh
```
