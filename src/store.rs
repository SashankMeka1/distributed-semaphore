use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub struct Element {
    pub value: Box<RedisValue>,
    pub expires_at: Option<Instant>,
}

impl Element {
    pub fn new(value: RedisValue, ttl: Option<Duration>) -> Self {
        Self {
            value: Box::new(value),
            expires_at: ttl.map(|d| Instant::now() + d),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |t| Instant::now() >= t)
    }
}

pub enum RedisValue {
    String(String),
    List(VecDeque<Element>),
    Hash(HashMap<String, Element>),
    Set(HashMap<String, Element>),
    ZSet(HashMap<String, Element>), // Element.value = String(score)
}

impl RedisValue {
    // Sweep expired elements inside a collection
    pub fn sweep(&mut self) {
        match self {
            RedisValue::List(deque) => deque.retain(|e| !e.is_expired()),
            RedisValue::Hash(map) => map.retain(|_, e| !e.is_expired()),
            RedisValue::Set(map) => map.retain(|_, e| !e.is_expired()),
            RedisValue::ZSet(map) => map.retain(|_, e| !e.is_expired()),
            RedisValue::String(_) => {}
        }
    }
}

pub struct Entry {
    pub value: RedisValue,
    pub expires_at: Option<Instant>,
}

impl Entry {
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |t| Instant::now() >= t)
    }
}

pub struct Store {
    data: HashMap<String, Entry>,
}

impl Store {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn set(&mut self, key: String, value: RedisValue, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        self.data.insert(key, Entry { value, expires_at });
    }

    pub fn get(&mut self, key: &str) -> Option<&RedisValue> {
        if let Some(entry) = self.data.get(key) {
            if entry.is_expired() {
                self.data.remove(key);
                return None;
            }
        }
        self.data.get(key).map(|e| &e.value)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut RedisValue> {
        if let Some(entry) = self.data.get(key) {
            if entry.is_expired() {
                self.data.remove(key);
                return None;
            }
        }
        self.data.get_mut(key).map(|e| &mut e.value)
    }

    pub fn del(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn exists(&mut self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn expire(&mut self, key: &str, ttl: Duration) -> bool {
        if let Some(entry) = self.data.get_mut(key) {
            if entry.is_expired() {
                self.data.remove(key);
                return false;
            }
            entry.expires_at = Some(Instant::now() + ttl);
            return true;
        }
        false
    }

    pub fn ttl_secs(&mut self, key: &str) -> i64 {
        match self.data.get(key) {
            None => -2,
            Some(entry) if entry.is_expired() => {
                self.data.remove(key);
                -2
            }
            Some(Entry { expires_at: None, .. }) => -1,
            Some(Entry { expires_at: Some(t), .. }) => {
                let remaining = t.saturating_duration_since(Instant::now());
                remaining.as_secs() as i64
            }
        }
    }

    pub fn flush(&mut self) {
        self.data.clear();
    }

    pub fn dbsize(&self) -> usize {
        self.data.len()
    }

    // Level 1: sweep expired top-level keys
    // Level 2: sweep expired elements inside surviving collections
    pub fn evict_expired(&mut self) {
        self.data.retain(|_, entry| !entry.is_expired());
        for entry in self.data.values_mut() {
            entry.value.sweep();
        }
    }
}
