#![no_std]
#![no_main]

mod node_one;
mod node_two_blocking;

//mod example;

use panic_probe as _;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_mcxa::{config::Config, peripherals};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_mcxa::init(Config::default());

    spawner.spawn(node_one::main(spawner, node_one::Resources {
        can: p.CAN0,
        tx_pin: p.P1_11,
        rx_pin: p.P1_2,
    }).expect("Failed to spawn node_one."));


    spawner.spawn(node_two_blocking::main(spawner, node_two_blocking::Resources {
        can: p.CAN1,
        tx_pin: p.P1_12,
        rx_pin: p.P1_17,
    }).expect("Failed to spawn node_two."));
}