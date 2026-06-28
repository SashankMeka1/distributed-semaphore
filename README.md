# distributed-semaphore

A Redis clone built in Rust, with a custom ZSet implementation designed for distributed semaphore use cases.

## What it is

A from-scratch Redis-compatible server implementing the RESP protocol over TCP. The core idea: multiple app servers can coordinate shared state through a single in-memory store — the classic distributed systems problem that Redis solves cheaply.

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

## Supported Commands

**Server**
- `PING`, `ECHO`, `FLUSHALL`, `DBSIZE`

**Strings**
- `GET`, `SET [EX seconds] [PX ms]`, `DEL`, `EXISTS`, `INCR`, `EXPIRE`, `TTL`

**Lists**
- `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LRANGE`

**Hashes**
- `HSET`, `HGET`, `HDEL`, `HGETALL`

**Sets**
- `SADD`, `SREM`, `SMEMBERS`, `SISMEMBER`

**Sorted Sets (ZSet)**
- `ZADD key score member [EX seconds]`
- `ZREM`, `ZCARD`, `ZSCORE`, `ZRANGE`, `ZREMRANGEBYSCORE`

## Distributed Semaphore

The ZSet implementation has per-member expiry independent of the score. This makes it well suited for a distributed semaphore pattern:

```
# acquire: sweep expired holders, check count, add yourself
ZREMRANGEBYSCORE semaphore 0 <now>
ZCARD semaphore               # if below max, proceed
ZADD semaphore <score> <client_id> EX 30

# release
ZREM semaphore <client_id>
```

Expired holders are swept on every ZSet operation — no background cleanup needed. A crashed client's lock expires automatically.

## Running

```bash
cargo run
```

Then connect with any Redis client:

```bash
redis-cli -p 6379
```

## Status

Work in progress — TCP listener and expiry worker not yet wired up.
