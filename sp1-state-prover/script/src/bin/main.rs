use alloy_sol_types::SolType;
use clap::Parser;
use orderbook::{match_order, Order, OrderType, State, Trade};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use serde::{Serialize, Deserialize};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");


/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long, default_value = "20")]
    a: u32,

    #[clap(long, default_value = "20")]
    b: u32,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::new();

    //Generating the inputs and outputs
    let start_state = State{pending_ask_orders:vec![], pending_bid_orders: vec![], trades: vec![]};
    let mut transactions: Vec<Order> = vec![];
    transactions.push(Order{id:"123".to_string(), address: "123".to_string(), order_type:OrderType::Bid, price: 1.05, quantity: 1000});
    transactions.push(Order{id:"123".to_string(), address: "123".to_string(), order_type:OrderType::Ask, price: 1.05, quantity: 1000});

    let mut last_state = start_state.clone();
    for tx in transactions.iter(){
        last_state = match_order(last_state, tx.clone());
    }
    println!("{:?}", last_state);
    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&start_state);
    stdin.write(&transactions);
    stdin.write(&last_state);
    // println!("n: {}", args.n);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(FIBONACCI_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        // let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
        // let PublicValuesStruct { n, a, b } = decoded;
        // println!("n: {}", n);
        // println!("a: {}", a);
        // println!("b: {}", b);

        // let (expected_a, expected_b) = fibonacci_lib::fibonacci(n);
        // assert_eq!(a, expected_a);
        // assert_eq!(b, expected_b);
        // println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Output is :- {:?}", output.as_slice());
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(FIBONACCI_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
