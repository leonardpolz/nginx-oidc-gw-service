use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;

lazy_static! {
    pub static ref TOKEN_CACHE: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}
