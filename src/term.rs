use std::io;

use crate::game::GameState;
use crate::input::{GetInput, Input, TextInput};
use crate::renderer::Renderer;
use crate::tile::Tile;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::{cursor, style, terminal, ExecutableCommand, QueueableCommand};
use indexmap::IndexMap;
use std::io::Write;
use std::time::Duration;

pub struct Terminal {
    stdout: io::Stdout,
    pub tiles: IndexMap<String, Tile>,
}

impl Terminal {
    pub fn new(tiles: IndexMap<String, Tile>) -> Self {
        let stdout = io::stdout();

        Self { stdout, tiles }
    }
}

impl Renderer for Terminal {
    fn resize(&self) -> anyhow::Result<glam::U16Vec2> {
        let size = terminal::size()?;
        Ok(glam::u16vec2(size.0, size.1))
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        terminal::enable_raw_mode().unwrap();
        self.stdout.execute(cursor::Hide)?;
        Ok(())
    }

    fn quit(&mut self) -> anyhow::Result<()> {
        terminal::disable_raw_mode().unwrap();
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.execute(cursor::MoveTo(0, 0))?;
        self.stdout.execute(cursor::Show)?;
        self.stdout
            .execute(style::SetForegroundColor(style::Color::Reset))?;
        self.stdout
            .execute(style::SetBackgroundColor(style::Color::Reset))?;
        Ok(())
    }

    fn render(&mut self, state: &GameState) -> anyhow::Result<()> {
        let diff = (state.display.size / 2).as_i16vec2();
        let ipos = state.position.as_i16vec2();
        let start = glam::i16vec2(ipos.x - diff.x, ipos.y - diff.y);
        let end = glam::i16vec2(ipos.x + diff.x, ipos.y + diff.y);

        tracing::info!("{start:?}, {end:?}");

        self.stdout.queue(cursor::MoveTo(0, 0))?;

        self.stdout
            .queue(style::SetForegroundColor(style::Color::DarkRed))?;
        self.stdout.queue(style::Print(&format!(
            "HLTH: {} ",
            text_bar(state.health as u8, 160, false)
        )))?;
        self.stdout
            .queue(style::SetForegroundColor(style::Color::Red))?;

        self.stdout.queue(style::Print(&format!(
            "HUNG: {} ",
            text_bar(state.hunger as u8 / 2, 184, false)
        )))?;

        println!("\r\n{:?} {:?} {:?}", state.position, start, end);

        // self.queue(style::SetForegroundColor(style::Color::DarkRed))?;
        // self.queue(style::Print(&format!("NUTR: {}", text_bar(40, 40, false))))?;

        // self.queue(style::SetForegroundColor(style::Color::DarkYellow))?;
        // self.queue(style::Print(&format!("{}", text_bar(40, 40, false))))?;

        // self.queue(style::SetForegroundColor(style::Color::Cyan))?;
        // self.queue(style::Print(&format!("{}", text_bar(40, 40, false))))?;

        // self.queue(style::SetForegroundColor(style::Color::DarkGreen))?;
        // self.queue(style::Print(&format!("{}", text_bar(40, 40, false))))?;

        self.stdout
            .queue(style::SetForegroundColor(style::Color::Reset))?;

        self.stdout.queue(cursor::MoveTo(0, 2))?;
        for i in 0..state.display.size.y {
            for j in 0..state.display.size.x {
                let tile = state.display.data[i as usize][j as usize];
                self.tile(&tile)?;
            }
            self.stdout.queue(style::Print("\n\r"))?;
        }
        self.stdout
            .queue(style::SetBackgroundColor(style::Color::Reset))?;

        self.stdout.flush()?;
        Ok(())
    }

    fn tile(&mut self, tile: &Tile) -> anyhow::Result<()> {
        self.stdout
            .queue(style::SetBackgroundColor(style::Color::AnsiValue(
                tile.back,
            )))?;
        self.stdout
            .queue(style::SetForegroundColor(style::Color::AnsiValue(
                tile.fore,
            )))?;
        self.stdout.queue(style::Print(tile.char))?;
        Ok(())
    }

    fn get_tile_index(&self, name: &str) -> Option<usize> {
        self.tiles.get_index_of("name")
    }
}

impl GetInput for Terminal {
    fn get_input(&self) -> Input {
        if let Some(key_code) = term_input_helper() {
            match key_code {
                KeyCode::Char('q') => Input::Quit,
                KeyCode::Char('h') => Input::Left,
                KeyCode::Char('j') => Input::Down,
                KeyCode::Char('k') => Input::Up,
                KeyCode::Char('l') => Input::Right,
                KeyCode::Char('u') => Input::UpLeft,
                KeyCode::Char('i') => Input::UpRight,
                KeyCode::Char('n') => Input::DownLeft,
                KeyCode::Char('m') => Input::DownRight,
                KeyCode::Char('1') => Input::Number('1'),
                KeyCode::Char('2') => Input::Number('2'),
                KeyCode::Char('3') => Input::Number('3'),
                KeyCode::Char('4') => Input::Number('4'),
                KeyCode::Char('5') => Input::Number('5'),
                KeyCode::Char('6') => Input::Number('6'),
                KeyCode::Char('7') => Input::Number('7'),
                KeyCode::Char('8') => Input::Number('8'),
                KeyCode::Char('9') => Input::Number('9'),
                KeyCode::Char('0') => Input::Number('0'),
                KeyCode::Char(';') => Input::MenuPrev,
                KeyCode::Char('\'') => Input::MenuNext,
                KeyCode::Char('s') => Input::Select,
                KeyCode::Char('t') => Input::EnterText,
                _ => Input::None,
            }
        } else {
            Input::None
        }
    }

    fn get_text_input(&self) -> TextInput {
        if let Some(key_code) = term_input_helper() {
            match key_code {
                KeyCode::Char(c) => TextInput::Char(c),
                KeyCode::Backspace => TextInput::Backspace,
                KeyCode::Esc => TextInput::Exit,
                _ => TextInput::None,
            }
        } else {
            TextInput::None
        }
    }
}

fn term_input_helper() -> Option<KeyCode> {
    if poll(Duration::from_millis(50)).unwrap() {
        match read().unwrap() {
            Event::Key(key_event) => Some(key_event.code),
            _ => None,
        }
    } else {
        None
    }
}
fn text_bar(value: u8, end: u8, shaded: bool) -> String {
    let mut bar = "".to_string();
    for _i in 0..(value / 8) {
        bar.push(BLOCK_FULL);
    }
    if !shaded {
        match value % 8 {
            0 => {}
            1 => bar.push(BLOCK_1_8),
            2 => bar.push(BLOCK_2_8),
            3 => bar.push(BLOCK_3_8),
            4 => bar.push(BLOCK_4_8),
            5 => bar.push(BLOCK_5_8),
            6 => bar.push(BLOCK_6_8),
            7 => bar.push(BLOCK_7_8),
            _ => panic!(),
        }
    } else {
        todo!();
    }

    let cmp = (end - value) / 8;
    if cmp > 0 {
        for _i in 0..cmp {
            bar.push(' ');
        }
    }
    bar.push(BLOCK_END);

    bar
}

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
