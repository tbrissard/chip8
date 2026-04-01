use std::{
    fmt::Display,
    ops::{BitAnd, BitXorAssign, ShrAssign},
};

use num_traits::{Num, WrappingShl, WrappingShr};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, ToText},
    widgets::{Block, Paragraph, Widget},
};

pub(crate) const DIGITS: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];

#[derive(Debug, Default)]
pub(crate) struct Sprite {
    bytes: Vec<u8>,
}

impl Sprite {
    const MAX_LEN: usize = 15;
}

impl From<&[u8]> for Sprite {
    fn from(value: &[u8]) -> Self {
        let len = value.len().min(Self::MAX_LEN);
        Self {
            bytes: value.iter().copied().take(len).collect(),
        }
    }
}

pub(crate) type StandardScreen = Screen<u64, 32>;

#[derive(Debug)]
pub(crate) struct Screen<T, const N: usize> {
    pixels: [T; N],
}

impl<T, const N: usize> Default for Screen<T, N>
where
    T: Num + Copy,
{
    fn default() -> Self {
        Self {
            pixels: [T::zero(); N],
        }
    }
}

impl<T, const N: usize> Screen<T, N>
where
    T: Num + Copy + WrappingShl + WrappingShr + BitAnd<Output = T> + From<u8> + BitXorAssign,
{
    const PIXEL_ON: char = '█';
    const PIXEL_OFF: char = ' ';

    pub(super) const WIDTH: usize = size_of::<T>() * 8;
    pub(super) const HEIGHT: usize = N;

    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn clear(&mut self) {
        for p in &mut self.pixels {
            *p = T::zero()
        }
    }

    /// toggles pixels by XORing the sprite onto the screen, returns true if any pixel was "turned "off"
    pub(crate) fn write_sprite(&mut self, sprite: &Sprite, mut x: usize, mut y: usize) -> bool {
        x %= Self::WIDTH;
        let shift_bits = |byte: T| (byte << (Self::WIDTH - 8)) >> x;

        let mut turned_off = false;
        y %= Self::HEIGHT;
        for (sprite_byte, i) in sprite.bytes.iter().copied().zip(y..Self::HEIGHT) {
            let sprite_byte = shift_bits(sprite_byte.into());
            let row = self
                .pixels
                .get_mut(i)
                .expect("should never go out of bound");
            turned_off |= (*row & sprite_byte) != T::zero();
            *row ^= sprite_byte;
        }

        turned_off
    }

    fn line(&self, row: usize) -> String {
        let mut pixels = String::new();
        let mut mask = T::one() << (Self::WIDTH - 1);

        for _ in 0..Self::WIDTH {
            let pixel = if (self.pixels[row] & mask) != T::zero() {
                Self::PIXEL_ON
            } else {
                Self::PIXEL_OFF
            };
            pixels.push(pixel);
            mask = mask >> 1;
        }

        pixels
    }

    fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        for i in 0..N {
            lines.push(self.line(i));
        }

        lines
    }
}

impl<T, const N: usize> Display for Screen<T, N>
where
    T: Num
        + Copy
        + WrappingShl
        + WrappingShr
        + BitAnd<Output = T>
        + From<u8>
        + BitXorAssign
        + ShrAssign<i32>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for l in self.lines() {
            writeln!(f, "{l}",)?;
        }

        Ok(())
    }
}

impl Widget for &StandardScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("Display".bold());

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let pixels = self.to_text();

        Paragraph::new(pixels)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {

    use super::{DIGITS, Sprite, StandardScreen};

    fn create_screen() -> StandardScreen {
        StandardScreen::new()
    }

    #[test]
    fn sprite_from_slice_shorter_than_max_length() {
        let bytes = vec![1; 1];
        let sprite = Sprite::from(bytes.as_slice());
        assert_eq!(sprite.bytes, bytes);
    }

    #[test]
    fn sprite_from_slice_equal_to_max_length() {
        let bytes = vec![1; Sprite::MAX_LEN];
        let sprite = Sprite::from(bytes.as_slice());
        assert_eq!(sprite.bytes, bytes);
    }

    #[test]
    fn sprite_from_slice_longer_than_max_length() {
        let bytes = vec![1; Sprite::MAX_LEN + 1];
        let sprite = Sprite::from(bytes.as_slice());
        assert_eq!(sprite.bytes, vec![1; Sprite::MAX_LEN]);
    }

    fn create_sprite_digit_zero() -> Sprite {
        Sprite::from(DIGITS[0].as_slice())
    }

    #[test]
    fn screen_new_empty() {
        let screen = create_screen();
        assert!(screen.pixels.iter().all(|row| *row == 0));
    }

    #[test]
    fn clear_screen() {
        let mut screen = create_screen();

        for row in &mut screen.pixels {
            *row = 0x42;
        }
        assert!(screen.pixels.iter().all(|row| *row != 0));
        screen.clear();
        assert!(screen.pixels.iter().all(|row| *row == 0));
    }

    #[test]
    fn write_sprite_within_bounds() {
        let mut screen = StandardScreen::new();
        let sprite = create_sprite_digit_zero();

        let turned_off = screen.write_sprite(&sprite, 0, 0);
        assert!(!turned_off);
        #[rustfmt::skip]
        let pixels: [u64;32] = [
            0b1111000000000000000000000000000000000000000000000000000000000000,
            0b1001000000000000000000000000000000000000000000000000000000000000,
            0b1001000000000000000000000000000000000000000000000000000000000000,
            0b1001000000000000000000000000000000000000000000000000000000000000,
            0b1111000000000000000000000000000000000000000000000000000000000000,
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
        ];
        assert_eq!(screen.pixels, pixels);
    }

    #[test]
    fn write_sprite_cut_horizontally() {
        let mut screen = StandardScreen::new();
        let sprite = create_sprite_digit_zero();

        let turned_off = screen.write_sprite(&sprite, 62, 0);
        assert!(!turned_off);
        #[rustfmt::skip]
        let pixels: [u64;32] = [
            0b0000000000000000000000000000000000000000000000000000000000000011,
            0b0000000000000000000000000000000000000000000000000000000000000010,
            0b0000000000000000000000000000000000000000000000000000000000000010,
            0b0000000000000000000000000000000000000000000000000000000000000010,
            0b0000000000000000000000000000000000000000000000000000000000000011,
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
        ];
        assert_eq!(screen.pixels, pixels);
    }

    #[test]
    fn write_sprite_cut_vertically() {
        let mut screen = StandardScreen::new();
        let sprite = create_sprite_digit_zero();

        let turned_off = screen.write_sprite(&sprite, 0, 30);
        assert!(!turned_off);
        #[rustfmt::skip]
        let pixels: [u64;32] = [
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
            0b0000000000000000000000000000000000000000000000000000000000000000,
            0b0000000000000000000000000000000000000000000000000000000000000000,
            0b0000000000000000000000000000000000000000000000000000000000000000,
            0b1111000000000000000000000000000000000000000000000000000000000000,
            0b1001000000000000000000000000000000000000000000000000000000000000,
        ];
        assert_eq!(screen.pixels, pixels);
    }

    #[test]
    fn write_sprite_coords_out_bound() {
        let mut screen = StandardScreen::new();
        let sprite = create_sprite_digit_zero();

        let turned_off = screen.write_sprite(&sprite, 68, 36);
        assert!(!turned_off);
        #[rustfmt::skip]
        let pixels: [u64;32] = [
            0,0,0,0,
            0b0000111100000000000000000000000000000000000000000000000000000000,
            0b0000100100000000000000000000000000000000000000000000000000000000,
            0b0000100100000000000000000000000000000000000000000000000000000000,
            0b0000100100000000000000000000000000000000000000000000000000000000,
            0b0000111100000000000000000000000000000000000000000000000000000000,
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
        ];
        assert_eq!(screen.pixels, pixels);
    }

    #[test]
    fn write_sprite_collision() {
        let mut screen = StandardScreen::new();
        let sprite_1 = create_sprite_digit_zero();
        let sprite_2 = create_sprite_digit_zero();

        let turned_off = screen.write_sprite(&sprite_1, 0, 0);
        assert!(!turned_off);
        let turned_off = screen.write_sprite(&sprite_2, 3, 0);
        assert!(turned_off);
        #[rustfmt::skip]
        let pixels: [u64;32] = [
            0b1110111000000000000000000000000000000000000000000000000000000000,
            0b1000001000000000000000000000000000000000000000000000000000000000,
            0b1000001000000000000000000000000000000000000000000000000000000000,
            0b1000001000000000000000000000000000000000000000000000000000000000,
            0b1110111000000000000000000000000000000000000000000000000000000000,
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        ];
        assert_eq!(screen.pixels, pixels);
    }
}
