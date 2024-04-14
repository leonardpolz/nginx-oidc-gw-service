use crate::models::role::Role;
use getset::Getters;
use uuid::Uuid;

#[derive(Getters)]
#[getset(get = "pub")]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    roles: Vec<Role>,
}

impl User {
    pub fn new(id: Uuid, name: String, email: String, roles: Vec<Role>) -> User {
        User {
            id,
            name,
            email,
            roles,
        }
    }
}
