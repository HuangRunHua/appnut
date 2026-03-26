//! KvStore implementations for ShopFlux models.

use openerp_store::KvStore;
use openerp_types::*;

use crate::server::model::*;

impl KvStore for User {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:user:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(&self.username);
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}

impl KvStore for Shop {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:shop:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}

impl KvStore for Category {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:category:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}

impl KvStore for Product {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:product:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}

impl KvStore for CartItem {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:cart_item:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(format!(
                "{}:{}",
                self.user.resource_id(),
                self.product.resource_id()
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}

impl KvStore for Order {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:order:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}

impl KvStore for OrderItem {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:order_item:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
}

impl KvStore for Review {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:review:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
}

impl KvStore for Address {
    const KEY: Field = Self::id;
    fn kv_prefix() -> &'static str {
        "shop:address:"
    }
    fn key_value(&self) -> String {
        self.id.to_string()
    }
    fn before_create(&mut self) {
        if self.id.is_empty() {
            self.id = Id::new(uuid::Uuid::new_v4().to_string().replace('-', ""));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if self.created_at.is_empty() {
            self.created_at = DateTime::new(&now);
        }
        self.updated_at = DateTime::new(&now);
    }
    fn before_update(&mut self) {
        self.updated_at = DateTime::new(chrono::Utc::now().to_rfc3339());
    }
}
