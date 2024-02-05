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
    // logging
    let log_file = std::fs::File::create("log.txt").unwrap();
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_line_number(true)
        .with_writer(log_file)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        println!("{args:?}");
        return Ok(());
    }

    let mut stdout = io::stdout();
    let size = terminal::size()?;
    let mut level = Level::new(Pos::new(65, 129));

    let tiles = import_toml::<Tile>("res/tiles.toml");
    let entities = import_toml::<MapEntity>("res/entity.toml");

    level.data[5][5] = tiles.get_index_of("brick_wall").unwrap();
    level.data[6][5] = tiles.get_index_of("brick_wall").unwrap();
    level.data[7][5] = tiles.get_index_of("brick_wall").unwrap();
    println!("{:?}", level.size);
    for i in 0..(level.size.row) {
        for j in 0..(level.size.col) {
            level.data[i as usize][j as usize] = tiles.get_index_of("tile").unwrap();
        }
    }
    level.data[20][5] = tiles.get_index_of("water").unwrap();

    stdout.init()?;

    let mut state = GameState::init(level, tiles, entities);
    let things: Vec<String> = vec![
        "cheesesjdashjkldhaklsjdhajklshdklashdkajls".to_string(),
        "fire".to_string(),
    ];
    state.ui_menu((Pos::new(1, 1), Pos::new(20, 30)), &things[..], true);

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

struct Level {
    size: Pos,
    data: Vec<Vec<usize>>,
}

impl Level {
    fn new(size: Pos) -> Self {
        let mut rows: Vec<Vec<usize>> = Vec::new();
        rows.reserve_exact(size.row as usize);
        for _i in 0..size.row {
            let mut col: Vec<usize> = Vec::new();
            col.reserve_exact(size.col as usize);
            for _j in 0..size.col {
                col.push(0);
            }
            rows.push(col);
        }

        Self { size, data: rows }
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
        let start_row = state.position.row - 22 / 2;
        let end_row = state.position.row + 22 / 2;
        let start_col = state.position.col - 80 / 2;
        let end_col = state.position.col + 80 / 2;

        let start = Pos::new(start_row, start_col);
        let end = Pos::new(end_row, end_col);

        tracing::info!("{start:?}, {end:?}");

        self.queue(cursor::MoveTo(0, 0))?;

        self.queue(style::SetForegroundColor(style::Color::DarkRed))?;
        self.queue(style::Print(&format!(
            "HLTH: {} ",
            text_bar(state.health, 160, false)
        )))?;
        self.queue(style::SetForegroundColor(style::Color::Red))?;

        self.queue(style::Print(&format!(
            "HUNG: {} ",
            text_bar(state.hunger / 2, 184, false)
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

        self.queue(style::SetForegroundColor(style::Color::Reset))?;

        self.queue(cursor::MoveTo(0, 2))?;
        for (display_i, level_i) in (start.row..end.row).enumerate() {
            for (display_j, level_j) in (start.col..end.col).enumerate() {
                if let Some(char) = state.ui[display_i][display_j] {
                    self.queue(style::SetForegroundColor(style::Color::Reset))?;
                    self.queue(style::SetBackgroundColor(style::Color::Reset))?;
                    self.queue(style::Print(char))?;
                } else if level_i < 0
                    || level_j < 0
                    || state.level.size.row <= level_i
                    || state.level.size.col <= level_j
                {
                    self.queue(style::SetForegroundColor(style::Color::Reset))?;
                    self.queue(style::SetBackgroundColor(style::Color::Reset))?;
                    self.queue(style::Print(" "))?;
                } else {
                    let tile = state.level.data[level_i as usize][level_j as usize];
                    if level_i == state.position.row && level_j == state.position.col {
                        self.entity(&state.tiles[tile], state.entity.get("player").unwrap())?;
                    } else {
                        self.tile(&state.tiles[tile])?;
                    }
                }
            }
            self.queue(style::Print("\n\r"))?;
        }
        self.queue(style::SetBackgroundColor(style::Color::Reset))?;

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
    level: Level,
    ui: Vec<Vec<Option<char>>>,
    number: String,
    pub tiles: IndexMap<String, Tile>,
    entity: IndexMap<String, MapEntity>,
}

// attack types: stab, slash, smash
// STAB, SLSH, SMSH

impl GameState {
    fn init(
        level: Level,
        tiles: IndexMap<String, Tile>,
        entity: IndexMap<String, MapEntity>,
    ) -> Self {
        let mut rows: Vec<Vec<Option<char>>> = Vec::new();
        rows.reserve_exact(20);
        for _i in 0..22 {
            let mut col: Vec<Option<char>> = Vec::new();
            col.reserve_exact(80);
            for _j in 0..80 {
                col.push(None);
            }
            rows.push(col);
        }

        Self {
            health: 160,
            hunger: 255,
            quit: false,
            position: Pos::new(5, 5),
            ui: rows,
            level,
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
                || new_row >= self.level.size.row
                || new_col < 0
                || new_col >= self.level.size.col
                || !self.tiles[self.level.data[new_row as usize][new_col as usize]].r#move
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

    fn ui_menu(&mut self, size: (Pos, Pos), items: &[String], shrink: bool) {
        // size.0 top left
        // size.1 bottom right

        // items
        // let mut longest_line = 0;
        for (i, row) in ((size.0.row as usize + 1)..(size.1.row as usize)).enumerate() {
            let chars: Vec<char> = if i < items.len() {
                items[i].chars().collect()
            } else {
                // if shrink {
                //     break;
                // } else {
                Vec::new()
                // }
            };

            for (j, col) in ((size.0.col as usize + 1)..(size.1.col as usize)).enumerate() {
                if j < chars.len() {
                    self.ui[row][col] = Some(chars[j]);
                } else {
                    self.ui[row][col] = Some(' ');
                }
            }
        }

        self.ui[size.0.row as usize][size.0.col as usize] = Some(LINE_DOWN_RIGHT);
        self.ui[size.0.row as usize][size.1.col as usize] = Some(LINE_DOWN_LEFT);
        self.ui[size.1.row as usize][size.0.col as usize] = Some(LINE_UP_RIGHT);
        self.ui[size.1.row as usize][size.1.col as usize] = Some(LINE_UP_LEFT);
        for col in (size.0.col + 1)..size.1.col {
            self.ui[size.0.row as usize][col as usize] = Some(LINE_HORZ);
            self.ui[size.1.row as usize][col as usize] = Some(LINE_HORZ);
        }
        for row in (size.0.row + 1)..size.1.row {
            self.ui[row as usize][size.0.col as usize] = Some(LINE_VERT);
            self.ui[row as usize][size.1.col as usize] = Some(LINE_VERT);
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
