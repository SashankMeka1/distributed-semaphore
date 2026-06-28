use std::collections::VecDeque;
use crate::resp::RespValue;
use crate::store::{Element, RedisValue, Store};

pub fn lpush(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'lpush'".to_string());
    }
    let key = args[1].clone();
    if !store.exists(&key) {
        store.set(key.clone(), RedisValue::List(VecDeque::new()), None);
    }
    match store.get_mut(&key) {
        Some(RedisValue::List(list)) => {
            for val in args[2..].iter().rev() {
                list.push_front(Element::new(RedisValue::String(val.clone()), None));
            }
            RespValue::Integer(list.len() as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

pub fn rpush(args: &[String], store: &mut Store) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'rpush'".to_string());
    }
    let key = args[1].clone();
    if !store.exists(&key) {
        store.set(key.clone(), RedisValue::List(VecDeque::new()), None);
    }
    match store.get_mut(&key) {
        Some(RedisValue::List(list)) => {
            for val in &args[2..] {
                list.push_back(Element::new(RedisValue::String(val.clone()), None));
            }
            RespValue::Integer(list.len() as i64)
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Integer(0),
    }
}

pub fn lpop(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'lpop'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::List(list)) => {
            list.retain(|e| !e.is_expired());
            match list.pop_front() {
                Some(el) => match *el.value {
                    RedisValue::String(s) => RespValue::BulkString(Some(s)),
                    _ => RespValue::BulkString(None),
                },
                None => RespValue::BulkString(None),
            }
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::BulkString(None),
    }
}

pub fn rpop(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'rpop'".to_string());
    }
    match store.get_mut(&args[1]) {
        Some(RedisValue::List(list)) => {
            list.retain(|e| !e.is_expired());
            match list.pop_back() {
                Some(el) => match *el.value {
                    RedisValue::String(s) => RespValue::BulkString(Some(s)),
                    _ => RespValue::BulkString(None),
                },
                None => RespValue::BulkString(None),
            }
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::BulkString(None),
    }
}

pub fn lrange(args: &[String], store: &mut Store) -> RespValue {
    if args.len() != 4 {
        return RespValue::Error("ERR wrong number of arguments for 'lrange'".to_string());
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
        Some(RedisValue::List(list)) => {
            list.retain(|e| !e.is_expired());
            let len = list.len() as i64;
            let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
            let stop = if stop < 0 { (len + stop).max(-1) } else { stop.min(len - 1) } as usize;
            if start > stop || list.is_empty() {
                return RespValue::Array(Some(vec![]));
            }
            let items: Vec<RespValue> = list
                .iter()
                .skip(start)
                .take(stop - start + 1)
                .filter_map(|el| match el.value.as_ref() {
                    RedisValue::String(s) => Some(RespValue::BulkString(Some(s.clone()))),
                    _ => None,
                })
                .collect();
            RespValue::Array(Some(items))
        }
        Some(_) => RespValue::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()),
        None => RespValue::Array(Some(vec![])),
    }
}
