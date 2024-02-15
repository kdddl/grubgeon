use indexmap::IndexMap;

pub fn import_toml<T: serde::de::DeserializeOwned>(path: &str) -> IndexMap<String, T> {
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

#[derive(Clone, Debug)]
pub enum Quadtree<T: Clone> {
    Leaf(T),
    Stem(Box<[Quadtree<T>; 4]>),
}

impl<T: Clone> Quadtree<T> {
    pub fn new(item: T) -> Self {
        Quadtree::Leaf(item)
    }

    pub fn subdivide(&mut self) {
        match self {
            Quadtree::Stem(array) => {
                for item in array.iter_mut() {
                    item.subdivide();
                }
            }
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

    pub fn value(&self) -> Option<&T> {
        match self {
            Quadtree::Stem(_) => None,
            Quadtree::Leaf(value) => Some(value),
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            Quadtree::Leaf(_) => true,
            Quadtree::Stem(_) => false,
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
