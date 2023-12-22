use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, Storable};
use std::borrow::Cow;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;
pub type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Product {
    pub id: u64,
    pub product_name: String,
    pub description: String,
    pub seller: String,
    pub price: u64,
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub category: Option<String>,
}

impl Storable for Product {
    // Implement the `Storable` trait for serialization
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Product {
    const MAX_SIZE: u32 = 1024; // Maximum size for the serialized data
    const IS_FIXED_SIZE: bool = false; // Data size is not fixed
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct ProductPayload {
    pub product_name: String,
    pub description: String,
    pub price: u64,
    pub category: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub enum Error {
    NotFound { msg: String },
    InvalidPayload{errors: Vec<String>},
    NotSeller
}
