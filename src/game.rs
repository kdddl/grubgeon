use crate::renderer::{self, Display};
use crate::{
    input::{GetInput, Input},
    level::Level,
    renderer::Renderer,
    tile::Tile,
};
use indexmap::IndexMap;

pub struct GameState {
    pub inputs: Box<dyn GetInput>,
    pub display: Display,
    pub health: u16,
    pub hunger: u16, // action points lmao
    pub quit: bool,
    pub position: glam::U16Vec2,
    pub level: Level,
    pub number: String,
    pub tiles: IndexMap<String, Tile>,
}

impl GameState {
    pub fn init(
        renderer: &dyn Renderer,
        inputs: Box<dyn GetInput>,
        level: Level,
        tiles: IndexMap<String, Tile>,
    ) -> Self {
        let mut size = renderer.resize().unwrap();
        size.y -= 3;
        let display = Display::new(size);

        Self {
            display,
            inputs,
            health: 160,
            hunger: 255,
            quit: false,
            position: glam::u16vec2(5, 5),
            level,
            number: "".to_string(),
            tiles,
        }
    }

    pub fn update(&mut self) {
        if self.hunger == 0 {
            self.quit = true;
            return;
        }

        match self.inputs.get_input() {
            Input::Quit => self.quit = true,
            Input::Left => self.try_move(glam::i16vec2(-1, 0)),
            Input::Up => self.try_move(glam::i16vec2(0, -1)),
            Input::Down => self.try_move(glam::i16vec2(0, 1)),
            Input::Right => self.try_move(glam::i16vec2(1, 0)),
            Input::UpLeft => self.try_move(glam::i16vec2(-1, -1)),
            Input::UpRight => self.try_move(glam::i16vec2(-1, 1)),
            Input::DownLeft => self.try_move(glam::i16vec2(1, -1)),
            Input::DownRight => self.try_move(glam::i16vec2(1, 1)),
            Input::Number('1') => self.number.push('1'),
            Input::Number('2') => self.number.push('2'),
            Input::Number('3') => self.number.push('3'),
            Input::Number('4') => self.number.push('4'),
            Input::Number('5') => self.number.push('5'),
            Input::Number('6') => self.number.push('6'),
            Input::Number('7') => self.number.push('7'),
            Input::Number('8') => self.number.push('8'),
            Input::Number('9') => self.number.push('9'),
            Input::Number('0') => self.number.push('0'),
            _ => {}
        }
    }

    fn try_move(&mut self, delta: glam::I16Vec2) {
        for _i in 1..=self.number() as i16 {
            let new_y = self.position.y as i16 + delta.y;
            let new_x = self.position.x as i16 + delta.x;
            tracing::info!("{new_y}, {new_x}");

            if self.hunger == 0
                || new_y < 0
                || new_y >= self.level.size.y as i16
                || new_x < 0
                || new_x >= self.level.size.x as i16
                || !self.tiles[self.level.data[new_y as usize][new_x as usize]].r#move
            // TODO: fix this
            {
                break;
            }
            self.position = (self.position.as_i16vec2() + delta).as_u16vec2();
            self.hunger -= 1;
        }
    }

    fn number(&mut self) -> u16 {
        let result = match self.number.parse() {
            Err(_) => 1,
            Ok(value) => value,
        };
        self.number = "".to_string();
        result
    }

    // fn ui_menu(&mut self, size: (Pos, Pos), items: &[String], selectable: bool) {
    //     // size.0 top left
    //     // size.1 bottom right

    //     // items
    //     // let mut longest_line = 0;
    //     for (i, row) in ((size.0.row as usize + 1)..(size.1.row as usize)).enumerate() {
    //         let chars: Vec<char> = if i < items.len() {
    //             items[i].chars().collect()
    //         } else {
    //             // if shrink {
    //             //     break;
    //             // } else {
    //             Vec::new()
    //             // }
    //         };

    //         for (j, col) in ((size.0.col as usize + 1)..(size.1.col as usize)).enumerate() {
    //             if j < chars.len() {
    //                 self.ui[row][col] = Some(chars[j]);
    //             } else {
    //                 self.ui[row][col] = Some(' ');
    //             }
    //         }
    //     }

    //     self.ui[size.0.row as usize][size.0.col as usize] = Some(LINE_DOWN_RIGHT);
    //     self.ui[size.0.row as usize][size.1.col as usize] = Some(LINE_DOWN_LEFT);
    //     self.ui[size.1.row as usize][size.0.col as usize] = Some(LINE_UP_RIGHT);
    //     self.ui[size.1.row as usize][size.1.col as usize] = Some(LINE_UP_LEFT);
    //     for col in (size.0.col + 1)..size.1.col {
    //         self.ui[size.0.row as usize][col as usize] = Some(LINE_HORZ);
    //         self.ui[size.1.row as usize][col as usize] = Some(LINE_HORZ);
    //     }
    //     for row in (size.0.row + 1)..size.1.row {
    //         self.ui[row as usize][size.0.col as usize] = Some(LINE_VERT);
    //         self.ui[row as usize][size.1.col as usize] = Some(LINE_VERT);
    //     }
    // }
}
