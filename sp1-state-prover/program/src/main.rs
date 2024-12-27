//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use orderbook::{Order, State, match_order,PublicValuesStruct};
use serde::{Serialize, Deserialize};


pub fn main() {
    let mut curr_state: State = sp1_zkvm::io::read();
    let transactions : Vec<Order> = sp1_zkvm::io::read();
    let res_state: State = sp1_zkvm::io::read();

    for tx in transactions.iter(){
        curr_state = match_order(curr_state, tx.clone());
    }

    if(res_state == curr_state){
        sp1_zkvm::io::commit(&true);
    }else{
        sp1_zkvm::io::commit(&false);
    }
}
