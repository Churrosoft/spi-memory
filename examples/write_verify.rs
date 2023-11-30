//! An STM32L073 example that measures write and erase times
//!
//! The flash chip is connected to SPI1
//!

#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use hex_display::HexDisplayExt;
use w25q::series25::Flash;

use stm32l0xx_hal as hal;

use crate::hal::{pac, prelude::*, rcc::Config, spi::MODE_0};

#[entry]
fn main() -> ! {
    hprintln!("START");
    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = p.RCC.freeze(Config::hsi16());

    let gpiob = p.GPIOB.split(&mut rcc);

    let sck = gpiob.pb3;
    let miso = gpiob.pb4;
    let mosi = gpiob.pb5;
    let cs = gpiob.pb6.into_push_pull_output();

    let spi = p
        .SPI1
        .spi((sck, miso, mosi), MODE_0, 4_000_000.Hz(), &mut rcc);

    let mut flash = Flash::init(spi, cs).unwrap();
    let id = flash.read_jedec_id().unwrap();
    hprintln!("{:?}", id);
    let mut fail_count = 0u32;

    const ITERATIONS: usize = 100;
    for i in 0..ITERATIONS {
        let mut inb = [0u8; 512];
        let mut inb_copy = [0u8; 512];
        let mut outb = [0u8; 512];
        for (n, b) in inb.iter_mut().enumerate() {
            *b = (n + i) as u8;
        }
        inb_copy.copy_from_slice(&inb);
        const ADDR: u32 = 0x0000_0000;
        match i {
            0..=90 => {
                hprintln!("sector erase");
                flash.erase_sectors(ADDR, 1).unwrap();
            }
            91..=98 => {
                hprintln!("block erase");
                flash.erase_block(ADDR).unwrap();
            },
            99..=100 => {
                hprintln!("chip erase");
                flash.erase_all().unwrap();
            },
            _ => (),
        }
        // inb will get overwritten below!
        flash.write_bytes(ADDR, &mut inb).unwrap();
        flash.read(ADDR, &mut outb).unwrap();
        if outb != inb_copy {
            hprintln!("Failed verification");
            // hprintln!("wrote: {}", inb_copy.hex());
            // hprintln!("read:  {}", outb.hex());
            fail_count += 1;
        }
        hprintln!("write iteration {}/{}", i, ITERATIONS);
    }

    if fail_count > 0 {
        hprintln!("num failures: {}", fail_count);
    }
    hprintln!("DONE");

    loop {}
}
