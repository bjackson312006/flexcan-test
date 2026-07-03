#![no_std]
#![no_main]

use panic_probe as _;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use embassy_mcxa::{
    bind_interrupts, config::Config, Peripherals, Peri,
    peripherals::{CAN0, P1_11, P1_2},
    flexcan::filter::{filters, Filter,},
    flexcan::classic::{
        FlexCan, FlexCanConfig, FlexCanRx, FlexCanTx, InterruptHandler,
        frame::{Frame, StandardId, ExtendedId},
    },
};

pub struct Resources {
    pub can:    Peri<'static, CAN0>,
    pub tx_pin: Peri<'static, P1_11>,
    pub rx_pin: Peri<'static, P1_2>,
}

bind_interrupts!(struct Irqs {
    CAN0 => InterruptHandler<CAN0>;
});

// Outgoing messages
const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).unwrap();
const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFAF).unwrap();

// Incoming messages
const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x02).unwrap();
const EXAMPLE_MESSAGE_FOUR: ExtendedId = ExtendedId::new(0x1232).unwrap();

#[embassy_executor::task]
pub async fn main(spawner: Spawner, resources: Resources) {
    // Create and configure a `FlexCan` instance for CAN0.
    let can0 = FlexCan::new(resources.can, resources.tx_pin, resources.rx_pin, FlexCanConfig {
        filters: filters!(
            Filter::Standard(EXAMPLE_MESSAGE_THREE), Filter::Extended(EXAMPLE_MESSAGE_FOUR),
        ),
        bitrate: 1_000_000,
        ..FlexCanConfig::default()
    }).expect("Failed to init FlexCan!!");

    // Split your `FlexCan` into separate `FlexCanTx` and `FlexCanRx` halves, and pass them to their respective tasks.
    let (tx0, rx0) = can0.split();
    spawner.spawn(can0_tx(tx0).expect("Failed to spawn `can0_tx()`."));
    spawner.spawn(can0_rx(rx0).expect("Failed to spawn `can0_rx()`."));
}

#[embassy_executor::task]
async fn can0_tx(mut tx0: FlexCanTx<'static>) {
    use core::sync::atomic::AtomicU16;
    use core::sync::atomic::Ordering;

    // Task for sending outgoing messages
    loop {
        static INCREMENTED: AtomicU16 = AtomicU16::new(0);
        let id = StandardId::new(INCREMENTED.load(Ordering::Relaxed)).unwrap();

        let frame1 = Frame::new_remote(id, 8).expect("Message payload too long!");
        let frame2 = Frame::new(EXAMPLE_MESSAGE_TWO, &[3, 4, 5, 6]).expect("Message payload too long!");
        tx0.send(&frame1).await;
        tx0.send(&frame2).await;

        INCREMENTED.fetch_add(1, Ordering::Relaxed);

        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn can0_rx(rx0: FlexCanRx<'static>) {
    // Task for receiving incoming messages
    loop {
        let frame = rx0.receive().await;
        defmt::info!("CAN0 RX id={:?} len={}", frame.id(), frame.dlc());
    }
}