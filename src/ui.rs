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
    fn get_focus(&self) -> bool;
    fn set_focus(&mut self, state: bool);
}

pub struct Menu {
    name: String,
    items: Vec<Vec<Tile>>,
    pub selection: usize,
    position: glam::U16Vec2,
    size: glam::U16Vec2,
    focus: bool,
}

impl Menu {
    pub fn new<S: Into<String>>(
        name: S,
        position: glam::U16Vec2,
        size: glam::U16Vec2,
        items: Vec<Vec<Tile>>,
    ) -> Self {
        Self {
            name: name.into(),
            position,
            items,
            size,
            selection: 0,
            focus: true,
        }
    }

    pub fn next(&mut self) {
        if self.selection < self.items.len() - 1 {
            self.selection += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        }
    }
}

impl Ui for Menu {
    fn get_focus(&self) -> bool {
        self.focus
    }

    fn set_focus(&mut self, state: bool) {
        self.focus = state;
    }

    fn render_to(&self, display: &mut Display) {
        let tile_void = Tile::new(' ', 0, 0, true);
        let name = Tile::from_string(&self.name, Some(15), Some(0));

        display[self.position] = Tile::new(LINE_DOWN_RIGHT, 15, 0, true);
        display[self.position + glam::u16vec2(self.size.x, 0)] =
            Tile::new(LINE_DOWN_LEFT, 15, 0, true);
        display[self.position + glam::u16vec2(0, self.size.y)] =
            Tile::new(LINE_UP_RIGHT, 15, 0, true);
        display[self.position + self.size] = Tile::new(LINE_UP_LEFT, 15, 0, true);
        for (index, col) in (1..self.size.x).enumerate() {
            if index < name.len() {
                display[self.position + glam::u16vec2(col, 0)] = name[index];
            } else {
                display[self.position + glam::u16vec2(col, 0)] = Tile::new(LINE_HORZ, 15, 0, true);
            }
            display[self.position + glam::u16vec2(col, self.size.y)] =
                Tile::new(LINE_HORZ, 15, 0, true);
        }
        for row in 1..self.size.y {
            display[self.position + glam::u16vec2(0, row)] = Tile::new(LINE_VERT, 15, 0, true);
            display[self.position + glam::u16vec2(self.size.x, row)] =
                Tile::new(LINE_VERT, 15, 0, true);
        }
        for (item_i, row) in (2..=self.size.y).enumerate() {
            if item_i < self.items.len() {
                let item = &self.items[item_i];
                for (index, col) in (2..=self.size.x).enumerate() {
                    const POINTER_OFFSET: usize = 2;
                    if index < POINTER_OFFSET {
                        display.data[row as usize][col as usize] = match index {
                            0 => {
                                if self.selection == item_i {
                                    Tile::new('>', 15, 0, true)
                                } else {
                                    tile_void
                                }
                            }
                            1 => tile_void,
                            _ => panic!(),
                        }
                    } else if index < item.len() + POINTER_OFFSET {
                        display.data[row as usize][col as usize] = item[index - 2];
                    } else {
                        display.data[row as usize][col as usize] = tile_void;
                    }
                }
            } else {
                for col in 2..=self.size.x {
                    display.data[row as usize][col as usize] = tile_void;
                }
            }
        }
    }
}
