use std::collections::HashMap;
use std::time::Duration;
use crate::resp::RespValue;
use crate::store::{Element, RedisValue, Store};

// ZADD key score member [EX seconds]
pub fn zadd(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 4 {
        return RespValue::Error("ERR wrong number of arguments for 'zadd'".to_string());
    }
    let score: f64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not a valid float".to_string()),
    };
    let member = args[3].clone();
    let mut ttl: Option<Duration> = None;

    if args.len() >= 6 && args[4].to_uppercase() == "EX" {
        match args[5].parse::<u64>() {
            Ok(secs) => ttl = Some(Duration::from_secs(secs)),
            Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
        }
    }

    let key = args[1].clone();
    if !store.exists(&key) {
        store.set(key.clone(), RedisValue::ZSet(HashMap::new()), None);
    }

    match store.get_mut(&key) {
        Some(RedisValue::ZSet(map)) => {
            map.retain(|_, e| !e.is_expired());
            let is_new = !map.contains_key(&member);
            map.insert(member, Element::new(RedisValue::String(score.to_string()), ttl));
            RespValue::Integer(if is_new { 1 } else { 0 })
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

// ZREM key member [member ...]
pub fn zrem(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'zrem'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::ZSet(map)) => {
            map.retain(|_, e| !e.is_expired());
            let count = args[2..].iter().filter(|m| map.remove(*m).is_some()).count();
            RespValue::Integer(count as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

// ZCARD key
pub fn zcard(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'zcard'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::ZSet(map)) => {
            map.retain(|_, e| !e.is_expired());
            RespValue::Integer(map.len() as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

// ZSCORE key member
pub fn zscore(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'zscore'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::ZSet(map)) => {
            map.retain(|_, e| !e.is_expired());
            match map.get(&args[2]) {
                Some(el) => match el.value.as_ref() {
                    RedisValue::String(s) => RespValue::BulkString(Some(s.clone())),
                    _ => RespValue::BulkString(None),
                },
                None => RespValue::BulkString(None),
            }
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::BulkString(None),
    }
}

// ZREMRANGEBYSCORE key min max
pub fn zremrangebyscore(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 4 {
        return RespValue::Error("ERR wrong number of arguments for 'zremrangebyscore'".to_string());
    }
    let min: f64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not a valid float".to_string()),
    };
    let max: f64 = match args[3].parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not a valid float".to_string()),
    };
    match store.get_mut(&args[1]) {
        Some(RedisValue::ZSet(map)) => {
            map.retain(|_, e| !e.is_expired());
            let before = map.len();
            map.retain(|_, el| match el.value.as_ref() {
                RedisValue::String(s) => {
                    let score: f64 = s.parse().unwrap_or(0.0);
                    score < min || score > max
                }
                _ => true,
            });
            RespValue::Integer((before - map.len()) as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

// ZRANGE key start stop
pub fn zrange(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 4 {
        return RespValue::Error("ERR wrong number of arguments for 'zrange'".to_string());
    }
    let start: i64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
    };
    let stop: i64 = match args[3].parse() {
        Ok(n) => n,
        Err(_) => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
    };
    match store.get_mut(&args[1]) {
        Some(RedisValue::ZSet(map)) => {
            map.retain(|_, e| !e.is_expired());
            let mut members: Vec<(&String, f64)> = map.iter()
                .filter_map(|(k, el)| match el.value.as_ref() {
                    RedisValue::String(s) => s.parse::<f64>().ok().map(|score| (k, score)),
                    _ => None,
                })
                .collect();
            members.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let len = members.len() as i64;
            let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
            let stop = if stop < 0 { (len + stop).max(-1) } else { stop.min(len - 1) } as usize;

            if start > stop || members.is_empty() {
                return RespValue::Array(Some(vec![]));
            }
            let items = members[start..=stop]
                .iter()
                .map(|(k, _)| RespValue::BulkString(Some(k.to_string())))
                .collect();
            RespValue::Array(Some(items))
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Array(Some(vec![])),
    }
}
