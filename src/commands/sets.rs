use std::collections::HashSet;
use crate::resp::RespValue;
use crate::store::{RedisValue, Store};

pub fn sadd(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'sadd'".to_string());
    }
    let key = args[1].clone();
    if !store.exists(&key) {
        store.set(key.clone(), RedisValue::Set(HashSet::new()), None);
    }
    match store.get_mut(&key) {
        Some(RedisValue::Set(set)) => {
            let count = args[2..].iter().filter(|v| set.insert((*v).clone())).count();
            RespValue::Integer(count as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

pub fn srem(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'srem'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::Set(set)) => {
            let count = args[2..].iter().filter(|v| set.remove(*v)).count();
            RespValue::Integer(count as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

pub fn smembers(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'smembers'".to_string());
    }
    match store.get(&args[1]) {
        Some(RedisValue::Set(set)) => {
            let items = set.iter()
                .map(|s| RespValue::BulkString(Some(s.clone())))
                .collect();
            RespValue::Array(Some(items))
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Array(Some(vec![])),
    }
}

pub fn sismember(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'sismember'".to_string());
    }
    match store.get(&args[1]) {
        Some(RedisValue::Set(set)) => RespValue::Integer(if set.contains(&args[2]) { 1 } else { 0 }),
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}
