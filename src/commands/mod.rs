mod strings;
mod lists;
mod hashes;
mod sets;
mod server;
mod zsets;

use crate::resp::RespValue;
use crate::store::Store;

pub fn dispatch(args: Vec<String>, store: &mut Store) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR empty command".to_string());
    }
    let cmd = args[0].to_uppercase();
    match cmd.as_str() {
        "PING"     => server::ping(&args),
        "ECHO"     => server::echo(&args),
        "FLUSHALL" => server::flushall(store),
        "DBSIZE"   => server::dbsize(store),

        "GET"    => strings::get(&args, store),
        "SET"    => strings::set(&args, store),
        "DEL"    => strings::del(&args, store),
        "EXISTS" => strings::exists(&args, store),
        "INCR"   => strings::incr(&args, store),
        "EXPIRE" => strings::expire(&args, store),
        "TTL"    => strings::ttl(&args, store),

        "LPUSH"  => lists::lpush(&args, store),
        "RPUSH"  => lists::rpush(&args, store),
        "LPOP"   => lists::lpop(&args, store),
        "RPOP"   => lists::rpop(&args, store),
        "LRANGE" => lists::lrange(&args, store),

        "HSET"    => hashes::hset(&args, store),
        "HGET"    => hashes::hget(&args, store),
        "HDEL"    => hashes::hdel(&args, store),
        "HGETALL" => hashes::hgetall(&args, store),

        "SADD"      => sets::sadd(&args, store),
        "SREM"      => sets::srem(&args, store),
        "SMEMBERS"  => sets::smembers(&args, store),
        "SISMEMBER" => sets::sismember(&args, store),

        "ZADD"             => zsets::zadd(&args, store),
        "ZREM"             => zsets::zrem(&args, store),
        "ZCARD"            => zsets::zcard(&args, store),
        "ZSCORE"           => zsets::zscore(&args, store),
        "ZRANGE"           => zsets::zrange(&args, store),
        "ZREMRANGEBYSCORE" => zsets::zremrangebyscore(&args, store),

        _ => RespValue::Error(format!("ERR unknown command '{}'", args[0])),
    }
}
