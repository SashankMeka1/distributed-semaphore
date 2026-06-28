use std::time::Duration;
use crate::resp::RespValue;
use crate::store::{RedisValue, Store};

pub fn get(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'get'".to_string());
    }
    match store.get(&args[1]) {
        Some(RedisValue::String(s)) => RespValue::BulkString(Some(s.clone())),
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::BulkString(None),
    }
}

pub fn set(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'set'".to_string());
    }
    let mut ttl: Option<Duration> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].to_uppercase().as_str() {
            "EX" => {
                i += 1;
                if i >= args.len() {
                    return RespValue::Error("ERR syntax error".to_string());
                }
                match args[i].parse::<u64>() {
                    Ok(secs) => ttl = Some(Duration::from_secs(secs)),
                    Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
                }
            }
            "PX" => {
                i += 1;
                if i >= args.len() {
                    return RespValue::Error("ERR syntax error".to_string());
                }
                match args[i].parse::<u64>() {
                    Ok(ms) => ttl = Some(Duration::from_millis(ms)),
                    Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
                }
            }
            _ => return RespValue::Error("ERR syntax error".to_string()),
        }
        i += 1;
    }
    store.set(args[1].clone(), RedisValue::String(args[2].clone()), ttl);
    RespValue::SimpleString("OK".to_string())
}

pub fn del(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'del'".to_string());
    }
    let count = args[1..].iter().filter(|k| store.del(k)).count();
    RespValue::Integer(count as i64)
}

pub fn exists(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'exists'".to_string());
    }
    let count = args[1..].iter().filter(|k| store.exists(k)).count();
    RespValue::Integer(count as i64)
}

pub fn incr(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'incr'".to_string());
    }
    let key = &args[1];
    let current = match store.get(key) {
        None => 0i64,
        Some(RedisValue::String(s)) => match s.parse::<i64>() {
            Ok(n) => n,
            Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
        },
        Some(_) => return RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
    };
    let next = current + 1;
    store.set(key.clone(), RedisValue::String(next.to_string()), None);
    RespValue::Integer(next)
}

pub fn expire(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'expire'".to_string());
    }
    let secs: u64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
    };
    let set = store.expire(&args[1], Duration::from_secs(secs));
    RespValue::Integer(if set { 1 } else { 0 })
}

pub fn ttl(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'ttl'".to_string());
    }
    RespValue::Integer(store.ttl_secs(&args[1]))
}
