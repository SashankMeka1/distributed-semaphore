use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

pub struct ZSetEntry {
    pub score: f64,
    pub expires_at: Option<Instant>,
}

impl ZSetEntry {
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |t| Instant::now() >= t)
    }
}

pub enum RedisValue {
    String(String),
    List(VecDeque<String>),
    Hash(HashMap<String, String>),
    Set(HashSet<String>),
    ZSet(HashMap<String, ZSetEntry>),
}

struct Entry {
    value: RedisValue,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
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

    /// Sweep expired keys — called by background task
    pub fn evict_expired(&mut self) {
        self.data.retain(|_, entry| !entry.is_expired());
    }
}
