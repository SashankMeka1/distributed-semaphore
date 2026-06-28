use std::collections::HashMap;
use crate::resp::RespValue;
use crate::store::{RedisValue, Store};

pub fn hset(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 4 || args.len() % 2 != 0 {
        return RespValue::Error("ERR wrong number of arguments for 'hset'".to_string());
    }
    let key = args[1].clone();
    if !store.exists(&key) {
        store.set(key.clone(), RedisValue::Hash(HashMap::new()), None);
    }
    match store.get_mut(&key) {
        Some(RedisValue::Hash(map)) => {
            let mut count = 0;
            for pair in args[2..].chunks(2) {
                let inserted = !map.contains_key(&pair[0]);
                map.insert(pair[0].clone(), pair[1].clone());
                if inserted { count += 1; }
            }
            RespValue::Integer(count)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

pub fn hget(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'hget'".to_string());
    }
    match store.get(&args[1]) {
        Some(RedisValue::Hash(map)) => match map.get(&args[2]) {
            Some(val) => RespValue::BulkString(Some(val.clone())),
            None => RespValue::BulkString(None),
        },
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::BulkString(None),
    }
}

pub fn hdel(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'hdel'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::Hash(map)) => {
            let count = args[2..].iter().filter(|f| map.remove(*f).is_some()).count();
            RespValue::Integer(count as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

pub fn hgetall(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'hgetall'".to_string());
    }
    match store.get(&args[1]) {
        Some(RedisValue::Hash(map)) => {
            let items: Vec<RespValue> = map
                .iter()
                .flat_map(|(k, v)| {
                    vec![
                        RespValue::BulkString(Some(k.clone())),
                        RespValue::BulkString(Some(v.clone())),
                    ]
                })
                .collect();
            RespValue::Array(Some(items))
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Array(Some(vec![])),
    }
}
