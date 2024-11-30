use alloy_sol_types::sol;
use serde::{Serialize, Deserialize};

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 n;
        uint32 a;
        uint32 b;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct State{
    pub pending_bid_orders: Vec<Order>,
    pub pending_ask_orders: Vec<Order>,
    pub trades: Vec<Trade>
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Trade{
    pub ask_order: Order,
    pub bid_order: Order,
    pub price: f64,
    pub quantity: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Order{
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OrderType{
    Bid,
    Ask
}

pub fn match_order(curr_state: State, new_order: Order) -> State{

    if new_order.order_type==OrderType::Ask{
        let order_found = find_order(curr_state.pending_bid_orders, new_order.price);
        match order_found{
            Some(val) => {
                if(curr_state.pending_bid_orders[val].quantity == new_order.quantity){

                }else{
                    if(curr_state.pending_bid_orders[val].quantity > new_order.quantity){
                        
                    }else{

                    }
                }
            },
            None => 
        }
    }else{

    }

    return curr_state;
}

fn find_order(orders: Vec<Order>, price: f64) -> Option<usize>{

    for i in 0..orders.len(){
        if orders[i].price == price {
            return Some(i);
        }
        if(orders[i].price > price){
            return None;
        }
    }
    None
}