use std::collections::HashMap;
use crate::resp::RespValue;
use crate::store::{Element, RedisValue, Store};

pub fn sadd(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'sadd'".to_string());
    }
    let key = args[1].clone();
    if !store.exists(&key) {
        store.set(key.clone(), RedisValue::Set(HashMap::new()), None);
    }
    match store.get_mut(&key) {
        Some(RedisValue::Set(map)) => {
            let count = args[2..].iter().filter(|v| {
                let is_new = !map.contains_key(*v);
                map.insert((*v).clone(), Element::new(RedisValue::String((*v).clone()), None));
                is_new
            }).count();
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
        Some(RedisValue::Set(map)) => {
            let count = args[2..].iter().filter(|v| map.remove(*v).is_some()).count();
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
        Some(RedisValue::Set(map)) => {
            let items = map.iter()
                .filter(|(_, el)| !el.is_expired())
                .map(|(k, _)| RespValue::BulkString(Some(k.clone())))
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
        Some(RedisValue::Set(map)) => {
            let is_member = map.get(&args[2]).map_or(false, |el| !el.is_expired());
            RespValue::Integer(if is_member { 1 } else { 0 })
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}
