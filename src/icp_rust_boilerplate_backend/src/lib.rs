#[macro_use]
extern crate serde;
use ic_cdk::api::time;
use ic_cdk::{update, query, caller};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

mod types;
use types::*;
mod helpers;
use helpers::*;

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

// Function that allows users to fetch a product from the canister's storage
#[query]
fn get_product(product_id: u64) -> Result<Product, Error> {
    match _get_product(&product_id) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Product with product ID {} not found", product_id),
        }),
    }
}

// Function that allows users to add a product to the canister
#[update]
fn add_product(payload: ProductPayload) -> Result<Product, Error> {
    // checks the input data from the payload and return errors if any
    // otherwise continue function execution
    validate_product_payload(&payload)?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let product = Product {
        id,
        product_name: payload.product_name,
        description: payload.description,
        seller: caller().to_string(),
        price: payload.price,
        created_at: time(),
        updated_at: None,
        category: payload.category,
    };
    // save product
    do_insert(&product);
    Ok(product)
}

// Function that allows a seller of a product to update his product stored in the canister
#[update]
fn update_product_price(product_id: u64, price: u64) -> Result<Product, Error> {
    // check whether product exists
    match STORAGE.with(|service| service.borrow().get(&product_id)) {
        Some(mut product) => {
            // checks whether the caller is the product seller's principal
            is_caller_product_seller(&product)?;
            if price == 0{
                return Err(Error::InvalidPayload { errors: Vec::from([format!("New price must be greater than zero.")]) })
            }
            // update price and updated_at fields
            product.price = price;
            product.updated_at = Some(time());
            // save updated product
            do_insert(&product);
            Ok(product)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a product with product_id={}. Product not found",
                product_id
            ),
        }),
    }
}

fn do_insert(product: &Product) {
    STORAGE.with(|service| service.borrow_mut().insert(product.id, product.clone()));
}

// Function that allows a seller of a product to delete his product from the canister
#[update]
fn delete_product(product_id: u64) -> Result<Product, Error> {
    // check whether product exists and return product if true
    let product = get_product(product_id)?;
    // checks whether the caller is the product seller's principal
    is_caller_product_seller(&product)?;
    // delete product from the canister
    match STORAGE.with(|service| service.borrow_mut().remove(&product_id)) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a Product with product_id={}. product not found.",
                product_id
            ),
        }),
    }
}

// helper function to fetch a product from the canister's storage
fn _get_product(product_id: &u64) -> Option<Product> {
    STORAGE.with(|service| service.borrow().get(product_id))
}


// need this to generate candid
ic_cdk::export_candid!();