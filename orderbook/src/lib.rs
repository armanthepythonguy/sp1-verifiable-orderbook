use alloy_sol_types::sol;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint64 prevState;
        address[] traders;
        uint8[] orderTypes;
        uint256[] price;
        uint256[] quantity;
        uint64 newState;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Hash)]
pub struct State {
    pub pending_bid_orders: Vec<Order>,
    pub pending_ask_orders: Vec<Order>,
    pub trades: Vec<Trade>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Trade {
    pub id: String,
    pub ask_order: Order,
    pub bid_order: Order,
    pub price: f64,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Order {
    pub id: String,
    pub address: String,
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Hash)]
pub enum OrderType {
    Bid,
    Ask,
}

impl Hash for Order{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.address.hash(state);
        self.order_type.hash(state);
        // Hash f64 as its bit representation
        let normalized = if self.price == -0.0 { 0.0 } else { self.price };
        normalized.to_bits().hash(state);

        self.quantity.hash(state);
    }
}

impl Hash for Trade{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.ask_order.hash(state);
        self.bid_order.hash(state);
        // Hash f64 as its bit representation
        let normalized = if self.price == -0.0 { 0.0 } else { self.price };
        normalized.to_bits().hash(state);

        self.quantity.hash(state);
    }
}

pub fn match_order(mut curr_state: State, mut new_order: Order) -> State {
    match new_order.order_type {
        OrderType::Ask => {
            let mut pending_bid_orders = std::mem::take(&mut curr_state.pending_bid_orders);
            process_order(
                &mut curr_state,
                &mut new_order,
                &mut pending_bid_orders,
                OrderType::Ask,
            );
            curr_state.pending_bid_orders = pending_bid_orders;
        }
        OrderType::Bid => {
            let mut pending_ask_orders = std::mem::take(&mut curr_state.pending_ask_orders);
            process_order(
                &mut curr_state,
                &mut new_order,
                &mut pending_ask_orders,
                OrderType::Bid,
            );
            curr_state.pending_ask_orders = pending_ask_orders;
        }
    }

    curr_state
}

fn process_order(
    state: &mut State,
    new_order: &mut Order,
    matching_orders: &mut Vec<Order>,
    order_type: OrderType,
) {
    if let Some(index) = find_order(matching_orders, new_order.price) {
        let matched_order = &mut matching_orders[index];
        let trade_quantity = matched_order.quantity.min(new_order.quantity);

        let trade = Trade {
            id: format!("{}-{}", matched_order.id, new_order.id),
            ask_order: if order_type == OrderType::Ask {
                new_order.clone()
            } else {
                matched_order.clone()
            },
            bid_order: if order_type == OrderType::Bid {
                new_order.clone()
            } else {
                matched_order.clone()
            },
            price: new_order.price,
            quantity: trade_quantity,
        };

        state.trades.push(trade);

        if matched_order.quantity > trade_quantity {
            matched_order.quantity -= trade_quantity;
        } else {
            matching_orders.remove(index);
        }

        if new_order.quantity > trade_quantity {
            new_order.quantity -= trade_quantity;
            if order_type == OrderType::Ask {
                state.pending_ask_orders.push(new_order.clone());
                state.pending_ask_orders
                    .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
            } else {
                state.pending_bid_orders.push(new_order.clone());
                state.pending_bid_orders
                    .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
            }
        }
    } else {
        if order_type == OrderType::Ask {
            state.pending_ask_orders.push(new_order.clone());
            state.pending_ask_orders
                .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        } else {
            state.pending_bid_orders.push(new_order.clone());
            state.pending_bid_orders
                .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        }
    }
}


fn find_order(orders: &Vec<Order>, price: f64) -> Option<usize> {
    for (i, order) in orders.iter().enumerate() {
        if order.price == price {
            return Some(i);
        }
        if order.price > price {
            return None;
        }
    }
    None
}