use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::style;
use crossterm::{cursor, style::Print, terminal, ExecutableCommand, QueueableCommand};
use indexmap::IndexMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// default 80 x 24 window
// hunger as action points lmao???

fn import_toml<T: serde::de::DeserializeOwned>(path: &str) -> IndexMap<String, T> {
    let mut tiles: IndexMap<String, T> = IndexMap::new();
    let table: toml::Table = std::fs::read_to_string(path)
        .unwrap()
        .parse::<toml::Table>()
        .unwrap();
    let tile_names = table.keys().collect::<Vec<_>>();
    for tile_name in tile_names.into_iter() {
        let tile: T = toml::from_str(&toml::to_string(&table[tile_name]).unwrap()).unwrap();
        tiles.insert(tile_name.clone(), tile);
    }
    tiles
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        println!("{args:?}");
        return Ok(());
    }

    let mut stdout = io::stdout();
    let size = terminal::size()?;
    let mut display = DisplayWindow::new((80, 24 - 2));

    let tiles = import_toml::<Tile>("res/tiles.toml");
    let entities = import_toml::<MapEntity>("res/entity.toml");

    display.data[5][5] = tiles.get_index_of("brick_wall").unwrap();
    display.data[6][5] = tiles.get_index_of("brick_wall").unwrap();
    display.data[7][5] = tiles.get_index_of("brick_wall").unwrap();
    for i in 5..10 {
        for j in 5..10 {
            display.data[i][j] = tiles.get_index_of("tile").unwrap();
        }
    }
    display.data[20][5] = tiles.get_index_of("water").unwrap();

    stdout.init()?;

    let mut state = GameState::init(display, tiles, entities);

    loop {
        state.update();
        if state.quit {
            break;
        }
        stdout.render(&state)?;
    }

    stdout.quit()?;

    let mut dungeon = Quadtree::new(false);
    dungeon.expand();

    println!("{size:?}");
    println!("{:#?}", state.tiles);
    println!("{:#?}", state.entity);
    Ok(())
}

struct DisplayWindow {
    size: Pos,
    data: Vec<Vec<usize>>,
}

impl DisplayWindow {
    fn new(size: (u16, u16)) -> Self {
        let mut rows: Vec<Vec<usize>> = Vec::new();
        let size_p: (usize, usize) = (size.0 as usize, size.1 as usize);
        rows.reserve_exact(size_p.1);
        for _i in 0..size_p.1 {
            let mut col: Vec<usize> = Vec::new();
            col.reserve_exact(size_p.0);
            for _j in 0..size_p.0 {
                col.push(0);
            }
            rows.push(col);
        }

        Self {
            size: Pos {
                row: size.0 as i16,
                col: size.1 as i16,
            },
            data: rows,
        }
    }
}

trait TermOutput {
    fn init(&mut self) -> anyhow::Result<()>;
    fn quit(&mut self) -> anyhow::Result<()>;
    fn render(&mut self, state: &GameState) -> anyhow::Result<()>;
    fn tile(&mut self, tile: &Tile) -> anyhow::Result<()>;
    fn entity(&mut self, tile: &Tile, entity: &MapEntity) -> anyhow::Result<()>;
}

impl TermOutput for io::Stdout {
    fn init(&mut self) -> anyhow::Result<()> {
        self.execute(terminal::Clear(terminal::ClearType::All))?;
        terminal::enable_raw_mode().unwrap();
        self.execute(cursor::Hide)?;
        Ok(())
    }

    fn quit(&mut self) -> anyhow::Result<()> {
        terminal::disable_raw_mode().unwrap();
        self.execute(terminal::Clear(terminal::ClearType::All))?;
        self.execute(cursor::MoveTo(0, 0))?;
        self.execute(cursor::Show)?;
        self.execute(style::SetForegroundColor(style::Color::Reset))?;
        self.execute(style::SetBackgroundColor(style::Color::Reset))?;
        Ok(())
    }

    fn render(&mut self, state: &GameState) -> anyhow::Result<()> {
        self.queue(cursor::MoveTo(0, 0))?;

        self.queue(style::SetForegroundColor(style::Color::DarkRed))?;
        self.queue(style::Print(&format!(
            "HLTH: {} ",
            text_bar(state.health, 160, false)
        )))?;
        self.queue(style::SetForegroundColor(style::Color::Red))?;

        self.queue(style::Print(&format!(
            "HUNG: {}\r\n",
            text_bar(state.hunger, 184, false)
        )))?;

        self.queue(style::SetForegroundColor(style::Color::DarkRed))?;
        self.queue(style::Print(&format!(
            "                            NUTR: {}",
            text_bar(40, 40, false)
        )))?;

        self.queue(style::SetForegroundColor(style::Color::DarkYellow))?;
        self.queue(style::Print(&format!("{}", text_bar(40, 40, false))))?;

        self.queue(style::SetForegroundColor(style::Color::Cyan))?;
        self.queue(style::Print(&format!("{}", text_bar(40, 40, false))))?;

        self.queue(style::SetForegroundColor(style::Color::DarkGreen))?;
        self.queue(style::Print(&format!("{}", text_bar(40, 40, false))))?;

        self.queue(style::SetForegroundColor(style::Color::Reset))?;

        self.queue(cursor::MoveTo(0, 2))?;
        for i in 0..state.display.data.len() {
            for j in 0..state.display.data[i].len() {
                let tile = state.display.data[i][j];
                if i == (state.position.row as usize) && j == (state.position.col as usize) {
                    self.entity(&state.tiles[tile], state.entity.get("player").unwrap())?;
                } else {
                    self.tile(&state.tiles[tile])?;
                }
            }
            self.queue(style::Print("\n\r"))?;
        }
        self.execute(style::SetBackgroundColor(style::Color::Reset))?;

        self.flush()?;
        Ok(())
    }

    fn tile(&mut self, tile: &Tile) -> anyhow::Result<()> {
        self.queue(style::SetBackgroundColor(style::Color::AnsiValue(
            tile.back,
        )))?;
        self.queue(style::SetForegroundColor(style::Color::AnsiValue(
            tile.fore,
        )))?;
        self.queue(style::Print(tile.char))?;
        Ok(())
    }

    fn entity(&mut self, tile: &Tile, entity: &MapEntity) -> anyhow::Result<()> {
        self.queue(style::SetBackgroundColor(style::Color::AnsiValue(
            tile.back,
        )))?;
        self.queue(style::SetForegroundColor(style::Color::AnsiValue(
            entity.fore,
        )))?;
        self.queue(style::Print(entity.char))?;
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct MapEntity {
    r#char: char,
    fore: u8,
}

enum Item {
    BeefJerky,
}

struct GameState {
    health: u8,
    hunger: u8, // action points lmao
    quit: bool,
    position: Pos,
    display: DisplayWindow,
    number: String,
    pub tiles: IndexMap<String, Tile>,
    entity: IndexMap<String, MapEntity>,
}

// attack types: stab, slash, smash
// STAB, SLSH, SMSH

impl GameState {
    fn init(
        display: DisplayWindow,
        tiles: IndexMap<String, Tile>,
        entity: IndexMap<String, MapEntity>,
    ) -> Self {
        Self {
            health: 160,
            hunger: 160,
            quit: false,
            position: Pos::new(5, 5),
            display,
            number: "".to_string(),
            tiles,
            entity,
        }
    }

    fn update(&mut self) {
        if self.hunger == 0 {
            self.quit = true;
            return;
        }

        match term_input() {
            Input::Quit => self.quit = true,
            Input::MoveLeft => self.try_move(Pos::new(0, -1)),
            Input::MoveUp => self.try_move(Pos::new(-1, 0)),
            Input::MoveDown => self.try_move(Pos::new(1, 0)),
            Input::MoveRight => self.try_move(Pos::new(0, 1)),
            Input::MoveUpLeft => self.try_move(Pos::new(-1, -1)),
            Input::MoveUpRight => self.try_move(Pos::new(-1, 1)),
            Input::MoveDownLeft => self.try_move(Pos::new(1, -1)),
            Input::MoveDownRight => self.try_move(Pos::new(1, 1)),
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

    fn try_move(&mut self, delta: Pos) {
        for _i in 1..=self.number() as i16 {
            let row = delta.row;
            let col = delta.col;
            let new_row = self.position.row + row;
            let new_col = self.position.col + col;

            if self.hunger == 0
                || new_row < 0
                || new_row > self.display.size.col - 1
                || new_col < 0
                || new_col > self.display.size.row - 1
                || !self.tiles[self.display.data[new_row as usize][new_col as usize]].r#move
            // TODO: fix this
            {
                break;
            }
            self.position.row += row;
            self.position.col += col;
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
}

struct Pos {
    row: i16,
    col: i16,
}

impl Pos {
    fn new(row: i16, col: i16) -> Self {
        Self { row, col }
    }
}

#[derive(Copy, Clone, Debug)]
enum Input {
    MoveUp,
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUpLeft,
    MoveUpRight,
    MoveDownLeft,
    MoveDownRight,
    Inventory,
    None,
    Quit,
    Number(char),
}

#[derive(Debug, serde::Deserialize)]
struct Tile {
    r#char: char,
    fore: u8,
    back: u8,
    r#move: bool,
}

fn term_input() -> Input {
    if let Some(key_code) = term_input_helper() {
        match key_code {
            KeyCode::Char('h') => Input::MoveLeft,
            KeyCode::Char('j') => Input::MoveDown,
            KeyCode::Char('k') => Input::MoveUp,
            KeyCode::Char('l') => Input::MoveRight,
            KeyCode::Char('q') => Input::Quit,
            KeyCode::Char('u') => Input::MoveUpLeft,
            KeyCode::Char('i') => Input::MoveUpRight,
            KeyCode::Char('n') => Input::MoveDownLeft,
            KeyCode::Char('m') => Input::MoveDownRight,
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
            _ => Input::None,
        }
    } else {
        Input::None
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

#[derive(Clone, Debug)]
enum Quadtree<T: Clone> {
    Leaf(T),
    Stem(Box<[Quadtree<T>; 4]>),
}

impl<T: Clone> Quadtree<T> {
    fn new(item: T) -> Self {
        Quadtree::Leaf(item)
    }

    fn expand(&mut self) {
        match self {
            Quadtree::Stem(_) => {}
            Quadtree::Leaf(item) => {
                *self = Quadtree::Stem(Box::new([
                    self.clone(),
                    self.clone(),
                    self.clone(),
                    self.clone(),
                ]))
            }
        }
    }

    fn value(&self) -> Option<&T> {
        match self {
            Quadtree::Stem(_) => None,
            Quadtree::Leaf(value) => Some(value),
        }
    }
}

use std::ops::{Index, IndexMut};

impl<T: Clone> Index<usize> for Quadtree<T> {
    type Output = Quadtree<T>;

    fn index(&self, n: usize) -> &Self::Output {
        match self {
            Quadtree::Leaf(_) => self,
            Quadtree::Stem(array) => &array[n],
        }
    }
}

impl<T: Clone> IndexMut<usize> for Quadtree<T> {
    fn index_mut(&mut self, n: usize) -> &mut Self::Output {
        match self {
            Quadtree::Leaf(_) => self,
            Quadtree::Stem(array) => &mut array[n],
        }
    }
}
