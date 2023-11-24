#[macro_use]
extern crate serde;
use candid::{Decode, Encode, CandidType};
use ic_cdk::api::time;
use ic_cdk::{update, query};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Market {
    id: u64,
    product_name: String,
    product_id: u64,
    seller: String,
    price: u64,
    created_at: u64,
    updated_at: Option<u64>,
    categories: String,
}

impl Storable for Market {
    // Implement the `Storable` trait for serialization
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Market {
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

    static STORAGE: RefCell<StableBTreeMap<u64, Market, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

struct MarketPayload {
    product_name: String,
    product_id: u64,
    seller: String,
    price: u64,
    categories: String,
}

#[ic_cdk::query]
fn get_market(product_id: u64) -> Result<Market, Error> {
    match _get_market(&product_id) {
        Some(market) => Ok(market),
        None => Err(Error::NotFound {
            msg: format!("Market with Product ID {} not found", product_id),
        }),
    }
}

// create market
fn add_market(market: MarketPayload) -> Option<Market> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let market = Market {
        id,
        product_name: market.product_name,
        product_id: market.product_id,
        seller: market.seller,
        price: market.price,
        created_at: time(),
        updated_at: None,
        categories: market.categories,
    };

    do_insert(&market);
    Some(market)
}



// update market with the product_id created by the user

fn update_market(product_id: u64, payload: MarketPayload) -> Result<Market, Error> {
    match STORAGE.with(|service| service.borrow().get(&product_id)) {
        Some(mut market) => {
            market.seller = payload.seller;
            market.price = payload.price;
            market.updated_at = Some(time());
            do_insert(&market);
            Ok(market)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a market with product_id={}. market not found",
                product_id
            ),
        }),
    }
}




fn do_insert(market: &Market) {
    STORAGE.with(|service| service.borrow_mut().insert(market.id, market.clone()));
}


// delete market using the product_id created by the user
#[ic_cdk::update]
fn delete_market(product_id: u64) -> Result<Market, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&product_id)) {
        Some(market) => Ok(market),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a market with product_id={}. product not found.",
                product_id
            ),
        }),
    }
}


#[derive(CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

fn _get_market(product_id: &u64) -> Option<Market> {
    STORAGE.with(|service| service.borrow().get(product_id))
}


// need this to generate candid
ic_cdk::export_candid!();