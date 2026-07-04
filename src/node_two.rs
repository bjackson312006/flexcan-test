#![no_std]
#![no_main]

use panic_probe as _;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use static_cell::ConstStaticCell;

use embassy_mcxa::{
    bind_interrupts, config::Config, Peripherals, Peri,
    peripherals::{CAN1, P1_12, P1_17},
    flexcan::filter::{filters, Filter,},
    flexcan::classic::{
        FlexCan, FlexCanConfig, FlexCanRx, FlexCanTx, InterruptHandler, Async, RxQueue,
        frame::{Frame, StandardId, ExtendedId},
    },
};

pub struct Resources {
    pub can:    Peri<'static, CAN1>,
    pub tx_pin: Peri<'static, P1_12>,
    pub rx_pin: Peri<'static, P1_17>,
}

bind_interrupts!(struct Irqs {
    CAN1 => InterruptHandler<CAN1>;
});

static RX_QUEUE: ConstStaticCell<RxQueue<16>> = ConstStaticCell::new(RxQueue::new());

// Outgoing messages
const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x02).unwrap();
const EXAMPLE_MESSAGE_FOUR: ExtendedId = ExtendedId::new(0x1232).unwrap();

// Incoming messages
const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).unwrap();
const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFAF).unwrap();

#[embassy_executor::task]
pub async fn main(spawner: Spawner, resources: Resources) {
    // Create and configure a `FlexCan` instance for CAN1.
    let can1 = FlexCan::new_async(resources.can, resources.tx_pin, resources.rx_pin, RX_QUEUE.take(), FlexCanConfig {
        filters: filters!(
            Filter::AcceptAllStandard,
            //Filter::Standard(EXAMPLE_MESSAGE_ONE), Filter::Extended(EXAMPLE_MESSAGE_TWO),
        ),
        bitrate: 1_000_000,
        ..FlexCanConfig::default()
    }).expect("Failed to init FlexCan!!");

    // Split your `FlexCan` into separate `FlexCanTx` and `FlexCanRx` halves, and pass them to their respective tasks.
    let (tx1, rx1) = can1.split();
    spawner.spawn(can1_tx(tx1).expect("Failed to spawn `can1_tx()`."));
    spawner.spawn(can1_rx(rx1).expect("Failed to spawn `can1_rx()`."));
}

#[embassy_executor::task]
async fn can1_tx(mut tx1: FlexCanTx<'static, Async>) {
    // Task for sending outgoing messages
    loop {
        let frame1 = Frame::new(EXAMPLE_MESSAGE_THREE, &[0, 1, 2]).expect("Message payload too long!");
        let frame2 = Frame::new(EXAMPLE_MESSAGE_FOUR, &[3, 4, 5, 6]).expect("Message payload too long!");
        tx1.send(&frame1).await;
        tx1.send(&frame2).await;

        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn can1_rx(rx1: FlexCanRx<'static, Async>) {
    // Task for receiving incoming messages
    loop {
        let frame = rx1.receive().await;
        defmt::info!("CAN1 RX id={:?} len={}", frame.id(), frame.dlc());
    }
}