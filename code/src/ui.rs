use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{MonoTextStyleBuilder, ascii::FONT_5X8, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{self, Baseline, Text},
};

pub fn draw_menu<D>(
    display: &mut D,
    title: &str,
    info1: &str,
    info2: &str,
    main: [&str; 5],
    selected: Option<usize>,
) where
    D: DrawTarget<Color = BinaryColor>,
    D::Error: core::fmt::Debug,
{
    let big = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::Off)
        .build();

    let normal = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(BinaryColor::On)
        .build();

    let inverted = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(BinaryColor::Off)
        .build();

    //

    Rectangle::new(Point::new(0, 0), Size::new(128, 12))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(display)
        .unwrap();

    Text::with_baseline(title, Point::new(4, 1), big, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(info1, Point::new(64, 1), big, Baseline::Top)
        .draw(display)
        .unwrap();
    Text::with_baseline(info2, Point::new(98, 1), big, Baseline::Top)
        .draw(display)
        .unwrap();

    for (i, line) in main.iter().enumerate() {
        let y = 14 + (i as i32 * 10);

        let is_selected = selected == Some(i);
        if is_selected {
            Rectangle::new(Point::new(0, y - 1), Size::new(128, 10))
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(display)
                .unwrap();
        }

        let style = if is_selected { inverted } else { normal };

        Text::with_baseline(line, Point::new(4, y), style, Baseline::Top)
            .draw(display)
            .unwrap();
    }
}
