use crate::cell::{Cell, Flags};
use crate::text_buffer::TextBuffer;
use embedded_graphics::{
    mono_font::{
        MonoFont,
        // iso_8859_1::{FONT_9X18 as FONT, FONT_9X18_BOLD as FONT_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Drawable, Point},
    text::{Baseline, Text, TextStyle},
};

/// A [`TextBuffer`] on top of a frame buffer
///
/// The internal use [`embedded_graphics`] crate to render fonts to pixels.
///
/// The underlying frame buffer needs to implement `DrawTarget<Color = Rgb888>` trait
/// to draw pixels in RGB format.
pub struct TextOnGraphic<'a, D>
where
    D: DrawTarget,
{
    width: u32,
    height: u32,
    graphic: D,
    font: MonoFont<'a>,
    font_bold: Option<MonoFont<'a>>,
}

impl<'a, D> TextOnGraphic<'a, D>
where
    D: DrawTarget,
{
    /// Create a new text buffer on graphic.
    pub fn new(
        graphic: D,
        width: u32,
        height: u32,
        font: MonoFont<'a>,
        font_bold: Option<MonoFont<'a>>,
    ) -> Self {
        TextOnGraphic {
            width,
            height,
            graphic,
            font,
            font_bold,
        }
    }
}

impl<'a, D> TextBuffer for TextOnGraphic<'a, D>
where
    D: DrawTarget<Color = Rgb888>,
{
    #[inline]
    fn width(&self) -> usize {
        (self.width / self.font.character_size.width) as usize
    }

    #[inline]
    fn height(&self) -> usize {
        (self.height / self.font.character_size.height) as usize
    }

    fn read(&self, _row: usize, _col: usize) -> Cell {
        unimplemented!("reading char from graphic is unsupported")
    }

    #[inline]
    fn write(&mut self, row: usize, col: usize, cell: Cell) {
        if row >= self.height() || col >= self.width() {
            return;
        }
        let mut utf8_buf = [0u8; 8];
        let s = cell.c.encode_utf8(&mut utf8_buf);
        let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };

        // Build text style
        let mut style = MonoTextStyleBuilder::new()
            .text_color(fg.to_rgb())
            .background_color(bg.to_rgb());

        // Set font
        let font = if cell.flags.contains(Flags::BOLD)
            && let Some(font_bold) = &self.font_bold
        {
            font_bold
        } else {
            &self.font
        };
        style = style.font(font);

        // Apply optional styling
        if cell.flags.contains(Flags::STRIKEOUT) {
            style = style.strikethrough();
        }
        if cell.flags.contains(Flags::UNDERLINE) {
            style = style.underline();
        }

        // Draw
        let text = Text::with_text_style(
            s,
            Point::new(
                col as i32 * self.font.character_size.width as i32,
                row as i32 * self.font.character_size.height as i32,
            ),
            style.build(),
            TextStyle::with_baseline(Baseline::Top),
        );
        text.draw(&mut self.graphic).ok();
    }
}
