use indexmap::IndexMap;

use crate::level::Level;
use crate::level::RoomTile;
use crate::tile::Tile;
use crate::util::import_toml;
use std::io::Write;

fn to_room_size(n: u8) -> glam::U16Vec2 {
    let size = (2 << (n as u8)) + 1;
    glam::u16vec2(size, size)
}

fn from_room_size(size: glam::U16Vec2) -> u8 {
    (size.x as f64 - 1.0).log2() as u8
}

pub fn export(name: String, tiles: &IndexMap<String, Tile>, level: &Level) -> anyhow::Result<()> {
    // get room_size
    let size = from_room_size(level.size);

    let mut tiles_used = Vec::new();
    for i in 0..level.size.y {
        for j in 0..level.size.x {
            if !tiles_used.contains(&level.data[i as usize][j as usize]) {
                tiles_used.push(level.data[i as usize][j as usize])
            }
        }
    }
    tiles_used.sort();

    println!("hi, {tiles_used:?}");

    let mut data = level.data.clone();
    for i in 0..level.size.y {
        for j in 0..level.size.x {
            let index = tiles_used.binary_search(&data[i as usize][j as usize]);
            data[i as usize][j as usize] = index.unwrap();
        }
    }

    let tiles_used: Vec<String> = tiles_used
        .into_iter()
        .map(|x| tiles.get_index(x).unwrap().0.clone())
        .collect();

    let room = RoomTile {
        data,
        size,
        tiles: tiles_used,
    };

    let path = format!("res/room_size_{size}.toml");
    let mut rooms = import_toml::<RoomTile>(&path);

    // update if room with that name already exist else create it
    if rooms.contains_key(&name) {
        rooms[&name] = room;
    } else {
        rooms.insert(name, room);
    }

    println!("{rooms:?}");

    // write
    let mut file = std::fs::File::create(&path)?;
    let toml = toml::to_string(&rooms)?;
    file.write(toml.as_bytes())?;

    Ok(())
}

// let mut level = level::Level::new(glam::u16vec2(3, 3));
// let mut level = level::Level::new(glam::u16vec2(5, 5));
// let mut level = level::Level::new(glam::u16vec2(9, 9));
// let mut level = level::Level::new(glam::u16vec2(17, 17));
// let mut level = level::Level::new(glam::u16vec2(33, 33));
