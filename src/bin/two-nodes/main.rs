#![no_std]
#![no_main]

mod node_one;
mod node_two;

//mod example;

use panic_probe as _;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_mcxa::{config::Config, peripherals, Peri};
use assign_resources::assign_resources;

assign_resources! {
    node_one: NodeOneResources {
        can: CAN0,
        rx_pin: P1_11,
        tx_pin: P1_2,
    }
    node_two: NodeTwoResources {
        can: CAN1,
        rx_pin: P1_12,
        tx_pin: P1_17,
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_mcxa::init(Config::default());
    let r = split_resources!(p);

    spawner.spawn(node_one::main(spawner, r.node_one).expect("Failed to spawn node_one."));

    spawner.spawn(node_two::main(spawner, r.node_two).expect("Failed to spawn node_two."));
}