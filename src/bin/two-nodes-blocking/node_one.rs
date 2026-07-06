//! Example for FlexCAN Classic in Blocking mode.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
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

#[embassy_executor::task]
pub async fn main(_spawner: Spawner, resources: crate::NodeOneResources) {
    // Create and configure a `FlexCan` instance for CAN0.
    let mut can0 = FlexCan::new_blocking(
        resources.can,
        resources.rx_pin,
        resources.tx_pin,
        FlexCanConfig {
            filters: filters!(
                Filter::Standard(EXAMPLE_MESSAGE_THREE),
                Filter::Extended(EXAMPLE_MESSAGE_FOUR),
            ),
            bitrate: 1_000_000,
            ..FlexCanConfig::default()
        },
    )
    .expect("Failed to init FlexCan!!");

    loop {
        // Send outgoing messages.
        let frame1 = Frame::new(EXAMPLE_MESSAGE_ONE, &[0, 1, 2]).expect("Message payload too long!");
        let frame2 = Frame::new(EXAMPLE_MESSAGE_TWO, &[3, 4, 5, 6]).expect("Message payload too long!");
        can0.blocking_send(&frame1);
        can0.blocking_send(&frame2);

        // Drain any incoming messages.
        while let Ok(frame) = can0.blocking_receive_timeout(Duration::from_millis(1)) {
            defmt::info!("CAN0 RX id={:?} len={}", frame.id(), frame.dlc());
        }

        Timer::after(Duration::from_millis(500)).await;
    }
}
