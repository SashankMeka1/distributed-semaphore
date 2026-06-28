use crate::resp::RespValue;
use crate::store::Store;

pub fn ping(args: &[String]) -> RespValue {
    if args.len() > 1 {
        RespValue::BulkString(Some(args[1].clone()))
    } else {
        RespValue::SimpleString("PONG".to_string())
    }
}

pub fn echo(args: &[String]) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'echo'".to_string());
    }
    RespValue::BulkString(Some(args[1].clone()))
}

pub fn flushall(store: &mut Store) -> RespValue {
    store.flush();
    RespValue::SimpleString("OK".to_string())
}

pub fn dbsize(store: &mut Store) -> RespValue {
    RespValue::Integer(store.dbsize() as i64)
}
