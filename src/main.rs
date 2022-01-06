//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod ledstrip;
mod ws2812_pio;

use ws2812_pio::WS2812PIO;

use ledstrip::{LEDColor, LEDStrip};

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::gpio::{FunctionPio0, Pin};
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use pio;

use bsp::hal::pio::PIOExt;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut wd2812_strip: Pin<_, FunctionPio0> = pins.gpio8.into_mode();
    wd2812_strip.set_drive_strength(bsp::hal::gpio::OutputDriveStrength::TwoMilliAmps);

    let mut ws2812 = WS2812PIO::new(
        pac.PIO0,
        &mut pac.RESETS,
        clocks.system_clock.freq().integer() as f32,
        8,
    );

    let mut strip = LEDStrip::<2>::new();
    let mut a = 0;
    let mut dir: i32 = 1;

    loop {
        a = a + dir;

        strip.colors[0].r = a as u8;
        strip.colors[1].b = 32 - a as u8;

        if (a == 0) || (a == 32) {
            dir = -dir;
        }

        ws2812.output(&strip);

        delay.delay_ms(3);
    }
}

// End of file
