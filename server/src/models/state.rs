use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct State {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
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
    pub order_type: OrderType,
    pub price: f64,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OrderType {
    Bid,
    Ask,
}