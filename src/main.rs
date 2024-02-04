use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::style;
use crossterm::{cursor, style::Print, terminal, ExecutableCommand, QueueableCommand};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// default 80 x 24 window
// hunger as action points lmao???

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let size = terminal::size()?;
    let mut display = DisplayWindow::new((80, 24 - 2));
    stdout.init()?;

    let mut tiles: HashMap<String, CustomTile> = HashMap::new();
    let table: toml::Table = fs::read_to_string("res/tiles.toml")
        .unwrap()
        .parse::<toml::Table>()
        .unwrap();
    let tile_names = table.keys().collect::<Vec<_>>();
    for tile_name in tile_names.into_iter() {
        let tile: CustomTile =
            toml::from_str(&toml::to_string(&table[tile_name]).unwrap()).unwrap();
        tiles.insert(tile_name.clone(), tile);
    }

    // display.data[5][5] = Tile::Player;
    display.make_box(Pos::new(3, 3), Pos::new(3, 40));

    let mut state = GameState::init(display, tiles);

    loop {
        state.update();
        if state.quit {
            break;
        }
        stdout.render(&state, &state.display)?;
    }

    stdout.quit()?;

    let mut dungeon = Quadtree::new(false);
    dungeon.expand();

    for i in 0..4 {
        if rand::random() {
            dungeon[i].expand();
            for j in 0..4 {
                if rand::random() {
                    dungeon[i][j].expand();
                    for k in 0..4 {
                        if rand::random() {
                            dungeon[i][j][k].expand();
                            for l in 0..4 {
                                dungeon[i][j][k][l] = Quadtree::Leaf(rand::random());
                            }
                        } else {
                            dungeon[i][j][k] = Quadtree::Leaf(rand::random());
                        }
                    }
                } else {
                    dungeon[i][j] = Quadtree::Leaf(rand::random());
                }
            }
        } else {
            dungeon[i] = Quadtree::Leaf(rand::random());
        }
    }

    // println!("{dungeon:#?}");

    use std::fs;

    println!("{size:?}");
    println!("{:#?}", state.tiles);
    Ok(())
}

struct DisplayWindow {
    size: Pos,
    data: Vec<Vec<Tile>>,
}

impl DisplayWindow {
    fn new(size: (u16, u16)) -> Self {
        let mut rows: Vec<Vec<Tile>> = Vec::new();
        let size_p: (usize, usize) = (size.0 as usize, size.1 as usize);
        rows.reserve_exact(size_p.1);
        for _i in 0..size_p.1 {
            let mut col: Vec<Tile> = Vec::new();
            col.reserve_exact(size_p.0);
            for _j in 0..size_p.0 {
                col.push(Tile::Water);
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

    fn make_box(&mut self, pos: Pos, size: Pos) {
        for row in pos.row..(pos.row + size.row) {
            self.data[row as usize][pos.col as usize] = Tile::Wall;
            self.data[row as usize][(pos.col + size.col - 1) as usize] = Tile::Wall;
        }
        for col in pos.col..(pos.col + size.col) {
            self.data[pos.row as usize][col as usize] = Tile::Wall;
            self.data[(pos.row + size.row - 1) as usize][col as usize] = Tile::Wall;
        }
    }
}

trait TermOutput {
    fn init(&mut self) -> anyhow::Result<()>;
    fn quit(&mut self) -> anyhow::Result<()>;
    fn render(&mut self, state: &GameState, display: &DisplayWindow) -> anyhow::Result<()>;
    fn tile(&mut self, tile: &CustomTile) -> anyhow::Result<()>;
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

    fn render(&mut self, state: &GameState, display: &DisplayWindow) -> anyhow::Result<()> {
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
        for i in 0..display.data.len() {
            for j in 0..display.data[i].len() {
                let tile = tile_to_char(display.data[i][j]);
                // self.queue(style::SetBackgroundColor(tile.2))?;
                // self.queue(style::SetForegroundColor(tile.1))?;
                // if i as i16 == state.position.row && j as i16 == state.position.col {
                //     self.queue(style::SetForegroundColor(style::Color::Reset))?;
                //     self.queue(style::Print('@'))?;
                // } else {
                //     self.queue(style::Print(tile.0))?;
                // }
                self.tile(state.tiles.get("grass").unwrap());
            }
            self.queue(style::Print("\n\r"))?;
        }
        self.execute(style::SetBackgroundColor(style::Color::Reset))?;

        self.flush()?;
        Ok(())
    }

    fn tile(&mut self, tile: &CustomTile) -> anyhow::Result<()> {
        self.queue(style::SetBackgroundColor(style::Color::AnsiValue(
            tile.back,
        )))?;
        self.queue(style::SetForegroundColor(style::Color::AnsiValue(
            tile.fore,
        )))?;
        self.queue(style::Print(tile.char))?;
        Ok(())
    }
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
    tiles: HashMap<String, CustomTile>,
}

// attack types: stab, slash, smash
// STAB, SLSH, SMSH

impl GameState {
    fn init(display: DisplayWindow, tiles: HashMap<String, CustomTile>) -> Self {
        Self {
            health: 160,
            hunger: 160,
            quit: false,
            position: Pos::new(5, 5),
            display,
            number: "".to_string(),
            tiles,
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
                || new_col > self.display.size.row - 1 // TODO: fix this
                || !Self::valid(self.display.data[new_row as usize][new_col as usize])
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

    fn valid(tile: Tile) -> bool {
        match tile {
            Tile::Wall => false,
            Tile::Empty => true,
            _ => true,
        }
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

fn tile_to_char(tile: Tile) -> (char, style::Color, style::Color) {
    use style::Color::*;

    match tile {
        Tile::Empty => (' ', Reset, Reset),
        Tile::Wall => (BLOCK_FULL, Reset, Grey),
        Tile::Grass => (',', Rgb { r: 0, g: 150, b: 0 }, Rgb { r: 0, g: 100, b: 0 }),
        Tile::Water => ('~', Rgb { r: 0, g: 0, b: 150 }, Rgb { r: 0, g: 0, b: 100 }),
        Tile::Floor => (' ', Reset, Reset),
        Tile::Player => ('@', Reset, Reset),
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

#[derive(Copy, Clone, Debug)]
enum Tile {
    Empty,
    Wall,
    Grass,
    Floor,
    Player,
    Water,
}

#[derive(Debug, serde::Deserialize)]
struct CustomTile {
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
