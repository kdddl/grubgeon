use crate::game::GameState;
use crate::tile::Tile;
use crate::util::*;
use std::ops::{Index, IndexMut};

pub trait Renderer {
    fn init(&mut self) -> anyhow::Result<()>;
    fn quit(&mut self) -> anyhow::Result<()>;
    fn render(&mut self, state: &GameState) -> anyhow::Result<()>;
    fn tile(&mut self, tile: &Tile) -> anyhow::Result<()>;
    fn resize(&self) -> anyhow::Result<glam::U16Vec2>;
    fn get_tile_index(&self, name: &str) -> Option<usize>;
}

pub struct Display {
    pub size: glam::U16Vec2,
    pub data: Vec<Vec<Tile>>,
}

const TILE_VOID: Tile = Tile {
    r#char: ' ',
    fore: 0,
    back: 0,
    r#move: true,
};

impl Display {
    pub fn new(size: glam::U16Vec2) -> Self {
        let data: Vec<Vec<Tile>> = vec![vec![TILE_VOID; size.x as usize].clone(); size.y as usize];
        Self { size, data }
    }
}

impl Index<glam::U16Vec2> for Display {
    type Output = Tile;

    fn index(&self, index: glam::U16Vec2) -> &Self::Output {
        &self.data[index.y as usize][index.x as usize]
    }
}

impl IndexMut<glam::U16Vec2> for Display {
    fn index_mut(&mut self, index: glam::U16Vec2) -> &mut Self::Output {
        &mut self.data[index.y as usize][index.x as usize]
    }
}
