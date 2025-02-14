use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Getters)]
#[getset(get = "pub")]
pub struct OutboxRole {
    id: String,
    name: String,
}
