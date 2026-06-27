#![no_std]
#![no_main]

use panic_halt as _;
use embassy_executor::Spawner;
use embassy_mcxa::config::Config;
use embassy_mcxa::flexcan;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mcxa::init(Config::default());

    use embassy_mcxa::flexcan::filter::{Filter, FilterConfig, filters, StandardId, ExtendedId};
    use embassy_mcxa::flexcan::classic::{FlexCan, FlexCanConfig};
    use embassy_mcxa::peripherals::{CAN0, CAN1};
 
    const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).expect("Invalid ID (too large).");
    const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFFF).expect("Invalid ID (too large).");
    const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x100).expect("Invalid ID (too large).");
    const EXAMPLE_MESSAGE_THREE_MASK: StandardId = StandardId::new(0x7F0).expect("Invalid mask (too large).");

    let filters: FilterConfig = filters!(
        Filter::Standard(EXAMPLE_MESSAGE_ONE),
        Filter::Extended(EXAMPLE_MESSAGE_TWO),
        Filter::StandardMasked { id: EXAMPLE_MESSAGE_THREE, mask: EXAMPLE_MESSAGE_THREE_MASK },
    );

    let can = FlexCan::new(p.CAN0, p.P1_3, p.P1_2, FlexCanConfig {
        protocol_exception: true,
        filters: filters!(
            Filter::Standard(EXAMPLE_MESSAGE_ONE),
        )
    }).expect("Failed to init FlexCan!!");

    loop {}
}