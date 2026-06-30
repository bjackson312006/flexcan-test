#![no_std]
#![no_main]

#![allow(unused_imports)]

use panic_halt as _;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use embassy_mcxa::config::Config;
use embassy_mcxa::flexcan;
use embassy_mcxa::{bind_interrupts};
use embassy_mcxa::peripherals::{CAN0, CAN1};
use embassy_mcxa::flexcan::classic::{FlexCan, FlexCanConfig, FlexCanTx, FlexCanRx, InterruptHandler, BusErrorMode};
use embassy_mcxa::flexcan::classic::frame::{Frame, StandardId};
use embassy_mcxa::flexcan::filter::{filters, Filter, FilterConfig};
use embassy_mcxa::gpio::{Level, Output, DriveStrength, SlewRate};
use core::sync::atomic::{AtomicU32, Ordering};

const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).expect("Invalid ID (too large).");

bind_interrupts!(struct Irqs {
    CAN0 => InterruptHandler<CAN0>;
    CAN1 => InterruptHandler<CAN1>;
});

#[embassy_executor::task]
async fn can0_tx_task(mut tx0: FlexCanTx<'static>) {
    loop {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let frame = Frame::new(EXAMPLE_MESSAGE_ONE, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        tx0.send(&frame).await;

        match tx0.error_mode() {
            BusErrorMode::ErrorActive => defmt::info!("Current error mode is ErrorActive (normal operation)."),
            BusErrorMode::ErrorPassive => defmt::info!("Current error mode is ErrorPassive (some werid stuff starting to go on)."),
            BusErrorMode::BusOff => defmt::info!("Current error mode is BusOff (we have shit the bed)"),
            _ => defmt::info!("Unknown BusErrorMode returned??"),
        };

        defmt::info!("CAN0 sent frame with ID {}. COUNTER={}", frame.id(), COUNTER.load(Ordering::Relaxed));
        COUNTER.fetch_add(1, Ordering::Relaxed);
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn can0_rx_task(rx0: FlexCanRx<'static>) {
    // loop {
    //     let frame = rx0.receive().await;   // async, woken by the ISR
    //     defmt::info!("CAN0 RX id={:?} len={}", frame.id(), frame.dlc());
    // }
}

#[embassy_executor::task]
async fn can1_tx_task(mut tx1: FlexCanTx<'static>) {
    // let id = StandardId::new(0x123).unwrap();
    // loop {
    //     tx1.send(&frame).await;
    //     defmt::info!("CAN1 sent frame with ID {}", frame.id());
    //     Timer::after(Duration::from_millis(500)).await;
    // }
}

#[embassy_executor::task]
async fn can1_rx_task(rx1: FlexCanRx<'static>) {
    loop {
        defmt::info!("Entered can1_rx_task()");
        let frame = rx1.receive().await;
        defmt::info!("CAN1 RX id={:?} len={}", frame.id(), frame.dlc());
    }
}

#[embassy_executor::task]
async fn heartbeat() {
    loop {
        static COUNTER: AtomicU32 = AtomicU32::new(0); 
        defmt::info!("Heartbeat! COUNTER={}", COUNTER.load(Ordering::Relaxed));
        COUNTER.fetch_add(1, Ordering::Relaxed);

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_mcxa::init(Config::default());

    // config the CAN1 mode pin to be held low
    let can1_mode = Output::new(p.P3_22, Level::Low, DriveStrength::Normal, SlewRate::Slow);

    use embassy_mcxa::flexcan::filter::{Filter, FilterConfig, filters, StandardId, ExtendedId};
    use embassy_mcxa::flexcan::classic::{FlexCan, FlexCanConfig};

    let can0 = FlexCan::new(p.CAN0, p.P1_11, p.P1_2, FlexCanConfig {
        filters: filters!(
            Filter::Standard(EXAMPLE_MESSAGE_ONE),
        ),
        ..FlexCanConfig::default()
    }).expect("Failed to init FlexCan!!");
    let (tx0, rx0) = can0.split();

    let can1 = FlexCan::new(p.CAN1, p.P1_12, p.P1_17, FlexCanConfig {
        ..FlexCanConfig::default()
    }).expect("Failed to init FlexCan!!");
    let (tx1, rx1) = can1.split();

    spawner.spawn(can0_tx_task(tx0).expect("Failed to spawn `can0_tx_task()`."));
    spawner.spawn(can0_rx_task(rx0).expect("Failed to spawn `can0_rx_task()`."));
    spawner.spawn(can1_tx_task(tx1).expect("Failed to spawn `can1_tx_task()`."));
    spawner.spawn(can1_rx_task(rx1).expect("Failed to spawn `can1_rx_task()`."));
    spawner.spawn(heartbeat().expect("Failed to spawn `heartbeat()`."));

    loop {
        defmt::info!("is can1 enable pin set low: {}", &can1_mode.is_set_low());
        Timer::after(Duration::from_secs(1)).await;
    }
}