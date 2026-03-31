use std::sync::{Arc, Mutex};

use crate::models::{CreateUser, User};

struct Store {
    users: Vec<User>,
    next_id: u64,
}

#[derive(Clone)]
pub struct UserService {
    inner: Arc<Mutex<Store>>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Store {
                users: Vec::new(),
                next_id: 1,
            })),
        }
    }

    pub fn list(&self) -> Vec<User> {
        let store = self.inner.lock().expect("user store mutex poisoned");
        store.users.clone()
    }

    pub fn get(&self, id: u64) -> Option<User> {
        let store = self.inner.lock().expect("user store mutex poisoned");
        store.users.iter().find(|u| u.id == id).cloned()
    }

    pub fn create(&self, input: CreateUser) -> User {
        let mut store = self.inner.lock().expect("user store mutex poisoned");
        let id = store.next_id;
        store.next_id += 1;
        let user = User {
            id,
            name: input.name,
            email: input.email,
        };
        store.users.push(user.clone());
        user
    }
}
