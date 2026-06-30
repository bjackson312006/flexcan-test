#![no_std]
#![no_main]

use panic_halt as _;
use embassy_executor::{Spawner, Executor};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

use embassy_mcxa::config::Config;
use embassy_mcxa::flexcan;
use embassy_mcxa::{bind_interrupts};
use embassy_mcxa::peripherals::{CAN0, CAN1};
use embassy_mcxa::flexcan::classic::{FlexCan, FlexCanConfig, FlexCanTx, FlexCanRx, InterruptHandler};
use embassy_mcxa::flexcan::classic::frame::{Frame, StandardId};
use embassy_mcxa::flexcan::filter::{filters, Filter, FilterConfig};

bind_interrupts!(struct Irqs {
    CAN0 => InterruptHandler<CAN0>;
});

#[embassy_executor::task]
async fn can_tx_task(mut tx: FlexCanTx<'static>) {
    let id = StandardId::new(0x123).unwrap();
    loop {
        let frame = Frame::new(id, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        tx.send(&frame).await;            // async, completes via the ISR
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn can_rx_task(rx: FlexCanRx<'static>) {
    loop {
        let frame = rx.receive().await;   // async, woken by the ISR
        defmt::info!("RX id={:?} len={}", frame.id(), frame.dlc());
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mcxa::init(Config::default());

    use embassy_mcxa::flexcan::filter::{Filter, FilterConfig, filters, StandardId, ExtendedId};
    use embassy_mcxa::flexcan::classic::{FlexCan, FlexCanConfig};
 
    const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).expect("Invalid ID (too large).");

    let can = FlexCan::new(p.CAN0, p.P1_3, p.P1_2, FlexCanConfig {
        filters: filters!(
            Filter::Standard(EXAMPLE_MESSAGE_ONE),
        ),
        ..FlexCanConfig::default()
    }).expect("Failed to init FlexCan!!");
    let (tx, rx) = can.split();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner: Spawner| {
        spawner.spawn(can_tx_task(tx).expect("Failed to spawn `can_tx_task()`."));
        spawner.spawn(can_rx_task(rx).expect("Failed to spawn `can_rx_task()`"));
    });
}