//! Example for FlexCAN Classic in Blocking mode.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::config::Config;
use embassy_mcxa::{Peri, peripherals::{CAN1, P1_12, P1_17},};
use embassy_mcxa::flexcan::classic::frame::{ExtendedId, Frame, StandardId};
use embassy_mcxa::flexcan::classic::{FlexCan, FlexCanConfig};
use embassy_mcxa::flexcan::filter::{Filter, filters};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Outgoing messages
const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).unwrap();
const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFAF).unwrap();

// Incoming messages
const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x02).unwrap();
const EXAMPLE_MESSAGE_FOUR: ExtendedId = ExtendedId::new(0x1232).unwrap();

pub struct Resources {
    pub can:    Peri<'static, CAN1>,
    pub tx_pin: Peri<'static, P1_12>,
    pub rx_pin: Peri<'static, P1_17>,
}

#[embassy_executor::task]
pub async fn main(spawner: Spawner, resources: Resources) {

    // Create and configure a `FlexCan` instance for CAN1.
    let mut can1 = FlexCan::new_blocking(
        resources.can,
        resources.tx_pin,
        resources.rx_pin,
        FlexCanConfig {
            filters: filters!(
                Filter::Standard(EXAMPLE_MESSAGE_ONE),
                Filter::Extended(EXAMPLE_MESSAGE_TWO),
            ),
            bitrate: 1_000_000,
            ..FlexCanConfig::default()
        },
    )
    .expect("Failed to init FlexCan!!");

    loop {
        // Send outgoing messages.
        let frame1 = Frame::new(EXAMPLE_MESSAGE_THREE, &[0, 1, 2]).expect("Message payload too long!");
        let frame2 = Frame::new(EXAMPLE_MESSAGE_FOUR, &[3, 4, 5, 6]).expect("Message payload too long!");
        can1.blocking_send(&frame1);
        can1.blocking_send(&frame2);

        // Drain any incoming messages.
        while let Ok(frame) = can1.try_receive() {
            defmt::info!("CAN1 RX id={:?} len={}", frame.id(), frame.dlc());
        }

        Timer::after(Duration::from_millis(500)).await;
    }
}
