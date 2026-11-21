#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use log::info;

use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal_smartled::{RmtSmartLeds, WS2812_TIMING, buffer_size, color_order};
use smart_leds::{RGB8, SmartLedsWrite};

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{
        MonoTextStyleBuilder,
        ascii::{FONT_6X10, FONT_7X13, FONT_8X13_BOLD},
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    spi::{
        Mode as SpiMode,
        master::{Config as SpiConfig, Spi},
    },
};
use oled_async::{builder::Builder, displays::ssd1309::Ssd1309_128_64, mode::GraphicsMode};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 2.0.0
    // generator parameters: -o esp32s3 -o alloc -o unstable-hal -o embassy -o esp-backtrace -o log -o zed -o esp

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    //

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO12)
    .with_mosi(peripherals.GPIO11)
    .into_async();

    let mut res = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO9, Level::Low, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO13, Level::High, OutputConfig::default());

    let spi_device = ExclusiveDevice::new(spi, cs, embassy_time::Delay).unwrap();
    let di = SPIInterface::new(spi_device, dc);
    let raw_disp = Builder::new(Ssd1309_128_64 {}).connect(di);
    let mut display: GraphicsMode<_, _> = raw_disp.into();

    let mut delay = Delay::new();
    display.reset(&mut res, &mut delay).unwrap();
    display.init().await.unwrap();
    display.flush().await.unwrap();

    //

    info!("Embassy initialized!");

    let _ = spawner;

    let freq = Rate::from_mhz(80);
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();
    let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Rgb>::new(
        WS2812_TIMING,
        rmt.channel0,
        peripherals.GPIO21,
        freq,
    )
    .unwrap();

    //

    display.flush().await.unwrap();

    let mut index: u64 = 0;

    led.write([RGB8::new(0, 20, 25)]).unwrap();

    let header_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13_BOLD)
        .text_color(BinaryColor::On)
        .build();

    let big_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X13)
        .text_color(BinaryColor::On)
        .build();

    let small_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        index += 1;
        info!("{}", index);
        display.clear();

        match index % 4 {
            0 => {
                Text::with_baseline("Loading", Point::new(32, 16), big_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();

                Rectangle::new(Point::new(40, 44), Size::new(16, 16))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut display)
                    .unwrap();
            }
            1 => {
                Text::with_baseline("Loading.", Point::new(32, 16), big_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();

                Rectangle::new(Point::new(56, 44), Size::new(16, 16))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut display)
                    .unwrap();
            }
            2 => {
                Text::with_baseline("Loading..", Point::new(32, 16), big_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();

                Rectangle::new(Point::new(72, 44), Size::new(16, 16))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut display)
                    .unwrap();
            }
            _ => {
                Text::with_baseline("Loading...", Point::new(32, 16), big_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();

                Rectangle::new(Point::new(56, 44), Size::new(16, 16))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(&mut display)
                    .unwrap();
            }
        }

        display.flush().await.unwrap();
        Timer::after(Duration::from_millis(250)).await;
    }
}
