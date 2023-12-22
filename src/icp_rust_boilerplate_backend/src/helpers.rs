use ic_cdk::caller;

use crate::types::*;


fn is_invalid_string_input(str_input: &str) -> bool {
    return str_input.trim().len() == 0;
}

pub fn validate_product_payload(payload: &ProductPayload) -> Result<(), Error>{
    let mut errors: Vec<String> = Vec::new();
    if is_invalid_string_input(&payload.product_name){
        errors.push(format!("Product name='{}' cannot be empty.", payload.product_name))
    }
    let is_description_empty = is_invalid_string_input(&payload.description);
    let is_description_descriptive: Vec<&str> = payload.description.trim().split(" ").collect();

    if is_description_empty || is_description_descriptive.len() < 2{
        errors.push(format!("Description='{}' needs to be descriptive.", payload.description))
    }
    let category = payload.category.is_some();
    if category && is_invalid_string_input(&payload.category.clone().unwrap_or_default()){
        errors.push(format!("If a category is provided, category='{}' cannot be an empty string.", payload.category.clone().unwrap_or_default()));
    }
    if payload.price == 0{
        errors.push(format!("Price must be greater than zero."))
    }
    if errors.is_empty(){
        Ok(())
    }else{
        return Err(Error::InvalidPayload { errors })
    } 
}


pub fn is_caller_product_seller(product: &Product) -> Result<(), Error>{
    if product.seller != caller().to_string(){
        return Err(Error::NotSeller)
    }else{
        Ok(())
    }
}