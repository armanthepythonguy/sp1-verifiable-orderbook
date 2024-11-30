use alloy_sol_types::sol;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 n;
        uint32 a;
        uint32 b;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct State{
    pub pending_bid_orders: Vec<Order>,
    pub pending_ask_orders: Vec<Order>,
    pub trades: Vec<Trade>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Trade{
    pub id: String,
    pub ask_order: Order,
    pub bid_order: Order,
    pub price: f64,
    pub quantity: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Order{
    pub id: String,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OrderType{
    Bid,
    Ask
}

pub fn match_order(mut curr_state: State, mut new_order: Order) -> State{

    if new_order.order_type==OrderType::Ask{
        let order_found = find_order(curr_state.clone().pending_bid_orders, new_order.clone().price);
        match order_found{
            Some(val) => {
                if(curr_state.pending_bid_orders[val].quantity == new_order.quantity){
                    let new_trade = Trade{
                        id: curr_state.pending_bid_orders[val].clone().id+&new_order.id,
                        ask_order: new_order.clone(),
                        bid_order: curr_state.pending_bid_orders[val].clone(),
                        price: new_order.price,
                        quantity: new_order.quantity
                    };
                    curr_state.trades.push(new_trade);
                    curr_state.pending_bid_orders.remove(val);
                }else{
                    if(curr_state.pending_bid_orders[val].quantity > new_order.quantity){
                        curr_state.pending_bid_orders[val].quantity -= new_order.quantity;
                        let new_trade = Trade{
                            id: curr_state.pending_bid_orders[val].clone().id+&new_order.id,
                            ask_order: new_order.clone(),
                            bid_order: curr_state.pending_bid_orders[val].clone(),
                            price: new_order.price,
                            quantity: new_order.quantity
                        };
                        curr_state.trades.push(new_trade);
                    }else{
                        let new_trade = Trade{
                            id: curr_state.pending_bid_orders[val].clone().id+&new_order.id,
                            ask_order: new_order.clone(),
                            bid_order: curr_state.pending_bid_orders[val].clone(),
                            price: new_order.price,
                            quantity: curr_state.pending_bid_orders[val].quantity
                        };
                        curr_state.trades.push(new_trade);
                        new_order.quantity -= curr_state.pending_bid_orders[val].quantity;
                        curr_state.pending_ask_orders.push(new_order);
                        curr_state.pending_ask_orders.sort_by(|a,b| a.price.partial_cmp(&b.price).unwrap_or(Ordering::Equal));
                        curr_state.pending_bid_orders.remove(val);
                    }
                }
            },
            None => {
                curr_state.pending_ask_orders.push(new_order);
                curr_state.pending_ask_orders.sort_by(|a,b| a.price.partial_cmp(&b.price).unwrap_or(Ordering::Equal));
            }
        }
    }else{
        let order_found = find_order(curr_state.pending_ask_orders.clone(), new_order.clone().price);
        match order_found{
            Some(val) => {
                if(curr_state.pending_ask_orders[val].clone().quantity == new_order.clone().quantity){
                    let new_trade = Trade{
                        id: new_order.clone().id+&curr_state.pending_bid_orders[val].id,
                        ask_order: curr_state.pending_ask_orders[val].clone(),
                        bid_order: new_order.clone(),
                        price: new_order.price,
                        quantity: new_order.quantity
                    };
                    curr_state.trades.push(new_trade);
                    curr_state.pending_ask_orders.remove(val);
                }else{
                    if(curr_state.pending_ask_orders[val].quantity > new_order.quantity){
                        curr_state.pending_ask_orders[val].quantity -= new_order.quantity;
                        let new_trade = Trade{
                            id: curr_state.pending_bid_orders[val].clone().id+&new_order.id,
                            ask_order: curr_state.pending_ask_orders[val].clone(),
                            bid_order: new_order.clone(),
                            price: new_order.price,
                            quantity: new_order.quantity
                        };
                        curr_state.trades.push(new_trade);
                    }else{
                        let new_trade = Trade{
                            id: curr_state.pending_bid_orders[val].clone().id+&new_order.id,
                            ask_order: curr_state.pending_ask_orders[val].clone(),
                            bid_order: new_order.clone(),
                            price: new_order.price,
                            quantity: curr_state.pending_ask_orders[val].quantity
                        };
                        curr_state.trades.push(new_trade);
                        new_order.quantity -= curr_state.pending_ask_orders[val].quantity;
                        curr_state.pending_bid_orders.push(new_order);
                        curr_state.pending_bid_orders.sort_by(|a,b| b.price.partial_cmp(&a.price).unwrap_or(Ordering::Equal));
                        curr_state.pending_ask_orders.remove(val);
                    }
                }
            },
            None => {
                curr_state.pending_bid_orders.push(new_order);
                curr_state.pending_bid_orders.sort_by(|a,b| b.price.partial_cmp(&a.price).unwrap_or(Ordering::Equal));
            }
        }
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