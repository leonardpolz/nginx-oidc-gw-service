use crate::data_models::outbox_action::OutboxAction;
use crate::data_models::outbox_role::OutboxRole;
use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Getters)]
#[getset(get = "pub")]
pub struct OutboxUser {
    id: Uuid,
    email: String,
    roles: Vec<OutboxRole>,
    action: OutboxAction,
}
