#[macro_use]
extern crate serde;
use candid::{Decode, Encode, CandidType, Principal};
use ic_cdk::api::{time, caller};
use ic_cdk::{update, query};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Product {
    id: u64,
    product_name: String,
    seller_principal: Principal,
    price: u64,
    created_at: u64,
    updated_at: Option<u64>,
    categories: String,
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

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Product, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ProductPayload {
    product_name: String,
    price: u64,
    categories: String,
}

#[query]
fn get_product(id: u64) -> Result<Product, Error> {
    match _get_product(&id) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Product with Product ID {} not found", id),
        }),
    }
}

// Function to create a new product
#[update]
fn add_product(product: ProductPayload) -> Option<Product> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let product = Product {
        id,
        product_name: product.product_name,
        seller_principal: caller(),
        price: product.price,
        created_at: time(),
        updated_at: None,
        categories: product.categories,
    };

    do_insert(&product);
    Some(product)
}



// Function to update product with the id created by the user
#[update]
fn update_product(id: u64, payload: ProductPayload, new_seller_principal: Principal) -> Result<Product, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut product) => {
            let can_update = is_caller_seller(&product);
            if can_update.is_err() {
                return Err(can_update.unwrap_err())
            }
            product.seller_principal = new_seller_principal;
            product.price = payload.price;
            product.updated_at = Some(time());
            do_insert(&product);
            Ok(product)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a product with id={}. product not found",
                id
            ),
        }),
    }
}

// Helper function to check whether the caller is the seller of a product
fn is_caller_seller(product: &Product) -> Result<(), Error> {
    if product.seller_principal.to_string() != caller().to_string() {
        Err(Error::NotSeller)
    }else{
        Ok(())
    }
}
// Helper function to save a product
fn do_insert(product: &Product) {
    STORAGE.with(|service| service.borrow_mut().insert(product.id, product.clone()));
}


// Function to delete product using the id created by the user
#[update]
fn delete_product(id: u64) -> Result<Product, Error> {
    let product = _get_product(&id);
    if product.is_none() {
        return Err(Error::NotFound { msg: format!("Product not found.") })
    }
    let can_delete = is_caller_seller(&product.unwrap());
    if can_delete.is_err() {
        return Err(can_delete.unwrap_err())
    }
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a product with id={}. product not found.",
                id
            ),
        }),
    }
}


#[derive(CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    NotSeller
}

fn _get_product(id: &u64) -> Option<Product> {
    STORAGE.with(|service| service.borrow().get(id))
}


// need this to generate candid
ic_cdk::export_candid!();
