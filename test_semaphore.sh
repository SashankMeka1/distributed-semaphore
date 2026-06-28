#!/usr/bin/env bash

HOST=127.0.0.1
PORT=6379
SEM_KEY="semaphore"
MAX_HOLDERS=3
TTL=10  # seconds before a slot auto-expires

redis() {
    redis-cli -h $HOST -p $PORT "$@"
}

acquire() {
    local client_id=$1

    # sweep expired holders
    redis ZREMRANGEBYSCORE $SEM_KEY 0 0

    local count=$(redis ZCARD $SEM_KEY)

    if [ "$count" -lt "$MAX_HOLDERS" ]; then
        redis ZADD $SEM_KEY 1.0 $client_id EX $TTL > /dev/null
        echo "[ACQUIRED] $client_id (holders: $((count + 1))/$MAX_HOLDERS)"
        return 0
    else
        redis RPUSH waitlist $client_id > /dev/null
        echo "[WAITING]  $client_id (semaphore full at $MAX_HOLDERS)"
        return 1
    fi
}

release() {
    local client_id=$1
    redis ZREM $SEM_KEY $client_id > /dev/null
    echo "[RELEASED] $client_id"

    # promote next waiter if any
    local next=$(redis LPOP waitlist)
    if [ -n "$next" ] && [ "$next" != "nil" ]; then
        redis ZADD $SEM_KEY 1.0 $next EX $TTL > /dev/null
        echo "[PROMOTED] $next from waitlist"
    fi
}

status() {
    local count=$(redis ZCARD $SEM_KEY)
    local waiting=$(redis LRANGE waitlist 0 -1)
    echo ""
    echo "--- Semaphore Status ---"
    echo "Holders ($count/$MAX_HOLDERS): $(redis ZRANGE $SEM_KEY 0 -1)"
    echo "Waitlist: $waiting"
    echo "------------------------"
    echo ""
}

# cleanup
redis FLUSHALL > /dev/null
echo "=== Distributed Semaphore Test (max=$MAX_HOLDERS, ttl=${TTL}s) ==="
echo ""

# fill up the semaphore
acquire "client-A"
acquire "client-B"
acquire "client-C"
status

# this one should go to the waitlist
acquire "client-D"
acquire "client-E"
status

# release one slot — next waiter should be promoted
release "client-A"
status

# simulate a crash — client-B never releases, wait for expiry
echo "[CRASH]    client-B crashed (slot will expire in ${TTL}s)"
echo "Sleeping ${TTL}s to let client-B expire..."
sleep $TTL

acquire "client-F"
status

echo "=== Done ==="
