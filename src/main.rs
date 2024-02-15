use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::style;
use crossterm::{cursor, style::Print, terminal, ExecutableCommand, QueueableCommand};
use indexmap::IndexMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
mod game;
mod input;
mod level;
mod renderer;
mod term;
mod tile;
mod ui;
mod util;
use input::GetInput;
use renderer::Renderer;
use ui::Ui;

use crate::tile::Tile;

// default 80 x 24 window
// hunger as action points lmao???

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
    let mut level = level::Level::new(glam::u16vec2(129, 65));

    let tiles = util::import_toml::<tile::Tile>("res/tiles.toml");

    println!("{:?}", level.size);
    for i in 0..(level.size.y) {
        for j in 0..(level.size.x) {
            level.data[i as usize][j as usize] = tiles.get_index_of("tile").unwrap();
        }
    }
    // 13 x 7   5
    // 2 x 2    4
    // 4 x 4    3
    // 8 x 8    2
    // 16 x 16  1
    // 32 x 32  0

    let mut renderer = term::Terminal::new(tiles.clone());
    let inputs = Box::new(term::Terminal::new(tiles.clone()));
    let mut state = crate::game::GameState::init(&renderer, inputs, level, tiles);

    let display_tiles: Vec<Vec<Tile>> = state
        .tiles
        .iter()
        .map(|(key, tile)| {
            let mut vec = vec![*tile, Tile::new(' ', 0, 0, false)];
            vec.extend(Tile::from_string(key, Some(15), Some(0)));
            vec
        })
        .collect();

    state.ui.push(ui::Menu::new(
        "Tiles",
        glam::u16vec2(1, 1),
        glam::u16vec2(30, 15),
        display_tiles,
    ));

    state.ui[0].next();
    state.ui[0].next();
    state
        .level
        .generate(state.tiles.get_index_of("brick_wall").unwrap());

    // *(state.ui[0].as_ref()).prev();

    renderer.init()?;

    loop {
        state.update();
        if state.quit {
            break;
        }
        // state.resize(renderer.resize());
        renderer.render(&state)?;
    }

    renderer.quit()?;

    println!("{size:?}");
    println!("{:#?}", state.tiles);
    Ok(())
}

// render
// game
// ui
//

// attack types: stab, slash, smash
// STAB, SLSH, SMSH
