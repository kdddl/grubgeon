use crate::renderer::*;
use crate::tile::Tile;

const BLOCK_FULL: char = '█';
const BLOCK_7_8: char = '▉';
const BLOCK_6_8: char = '▊';
const BLOCK_5_8: char = '▋';
const BLOCK_4_8: char = '▌';
const BLOCK_3_8: char = '▍';
const BLOCK_2_8: char = '▎';
const BLOCK_1_8: char = '▏';
const BLOCK_DARK: char = '▓';
const BLOCK_MEDIUM: char = '▒';
const BLOCK_LIGHT: char = '░';
const BLOCK_END: char = '▏';

const LINE_HORZ: char = '─';
const LINE_VERT: char = '│';
const LINE_DOWN_RIGHT: char = '┌';
const LINE_DOWN_LEFT: char = '┐';
const LINE_UP_RIGHT: char = '└';
const LINE_UP_LEFT: char = '┘';
const LINE_CROSS: char = '┼';
const LINE_VERT_RIGHT: char = '├';
const LINE_HORZ_DOWN: char = '┬';
const LINE_VERT_LEFT: char = '┤';
const LINE_HORZ_UP: char = '┴';

pub trait Ui {
    fn render_to(&self, display: &mut Display);
}

pub struct Menu {
    items: Vec<String>,
    selection: usize,
    position: glam::U16Vec2,
    size: glam::U16Vec2,
}

impl Menu {
    pub fn new(position: glam::U16Vec2, size: glam::U16Vec2, items: Vec<String>) -> Box<Self> {
        Box::new(Self {
            position,
            items,
            size,
            selection: 0,
        })
    }
}

impl Ui for Menu {
    fn render_to(&self, display: &mut Display) {
        display[self.position] = Tile::new(LINE_DOWN_RIGHT, 15, 0, true);
        display[self.position + glam::u16vec2(self.size.x, 0)] =
            Tile::new(LINE_DOWN_LEFT, 15, 0, true);
        display[self.position + glam::u16vec2(0, self.size.y)] =
            Tile::new(LINE_UP_RIGHT, 15, 0, true);
        display[self.position + self.size] = Tile::new(LINE_UP_LEFT, 15, 0, true);
        for col in 1..self.size.x {
            display[self.position + glam::u16vec2(col, 0)] = Tile::new(LINE_HORZ, 15, 0, true);
            display[self.position + glam::u16vec2(col, self.size.y)] =
                Tile::new(LINE_HORZ, 15, 0, true);
        }
        for row in 1..self.size.y {
            display[self.position + glam::u16vec2(0, row)] = Tile::new(LINE_VERT, 15, 0, true);
            display[self.position + glam::u16vec2(self.size.x, row)] =
                Tile::new(LINE_VERT, 15, 0, true);
        }
    }
}
