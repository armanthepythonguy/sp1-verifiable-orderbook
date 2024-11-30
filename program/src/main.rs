//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use fibonacci_lib::{fibonacci, PublicValuesStruct};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct State{
    pub party_a: u64,
    pub party_b: u64,
    pub party_c: u64
}


pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let mut curr_json: State = sp1_zkvm::io::read();
    let trans_json : State = sp1_zkvm::io::read();
    let res_json: State = sp1_zkvm::io::read();

    curr_json.party_a += trans_json.party_a;
    curr_json.party_b += trans_json.party_b;
    curr_json.party_c += trans_json.party_c;

    if(curr_json == res_json){
        sp1_zkvm::io::commit(&res_json);
    }else{
        sp1_zkvm::io::commit(&0);
    }

}
