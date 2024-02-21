pub struct Level {
    pub size: glam::U16Vec2,
    pub data: Vec<Vec<usize>>,
}

impl Level {
    pub fn new(size: glam::U16Vec2) -> Self {
        let data = vec![vec![0; size.x as usize].clone(); size.y as usize];

        Self { size, data }
    }
}

pub struct QuadtreeFlat<'a, T: Clone> {
    position: u32,
    data: &'a T,
}

impl<'a, T: Clone> QuadtreeFlat<'a, T> {
    pub fn new(pos: u32, data: &'a T) -> Self {
        QuadtreeFlat {
            position: pos,
            data,
        }
    }
}

use crate::util::Quadtree;

impl Level {
    pub fn generate(&mut self, tile: usize) {
        let mut level = [Quadtree::new(0), Quadtree::new(0)];

        quadtree_gen(&mut level[0], 5);
        quadtree_gen(&mut level[1], 5);

        println!("{level:#?}");
        let rooms = iter(&level[0], 0, 0);
        let rooms2 = iter(&level[1], 0, 0);
        for item in rooms.iter() {
            let binary = format!("{:010b}", item.position);
            let mut out = "".to_string();
            for (i, c) in binary.chars().enumerate() {
                if i % 2 == 0 && i != 0 {
                    out.push(' ');
                }
                out.push(c);
            }
            println!("{out}, {}", item.data);
        }

        for room in rooms {
            let mut position = glam::u16vec2(0, 0);
            for i in 0..=5 {
                let x = ((room.position >> (2 * i)) & 0b10) >> 1;
                let y = (room.position >> (2 * i)) & 0b01;
                position.x += (x * (32 >> i)) as u16;
                position.y += (y * (32 >> i)) as u16;
            }
            println!("{position:?}, {}", room.data);
            let size = glam::u16vec2(2 << room.data, 2 << room.data);
            self.make_room(position, size, tile);
        }
        for room in rooms2 {
            let mut position = glam::u16vec2(64, 0);
            for i in 0..=5 {
                let x = ((room.position >> (2 * i)) & 0b10) >> 1;
                let y = (room.position >> (2 * i)) & 0b01;
                position.x += (x * (32 >> i)) as u16;
                position.y += (y * (32 >> i)) as u16;
            }
            println!("{position:?}, {}", room.data);
            let size = glam::u16vec2(2 << room.data, 2 << room.data);
            self.make_room(position, size, tile);
        }
    }

    fn make_room(&mut self, pos: glam::U16Vec2, size: glam::U16Vec2, tile: usize) {
        // 2 x 2    0
        // 4 x 4    1
        // 8 x 8    2
        // 16 x 16  3
        // 32 x 32  4
        // 64 x 64  5 two of these
        // 2 << i

        let (size_x, size_y) = (size.x as usize, size.y as usize);
        let (pos_x, pos_y) = (pos.x as usize, pos.y as usize);

        for col in 0..=size_x {
            self.data[pos_y][pos_x + col] = tile;
            self.data[pos_y + size_y][pos_x + col] = tile;
        }
        for row in 0..=size_y {
            self.data[pos_y + row][pos_x] = tile;
            self.data[pos_y + row][pos_x + size_x] = tile;
        }
    }
}

const WEIGHTS: [f32; 6] = [0.0, 0.1, 0.2, 0.7, 0.9, 1.0];

pub fn quadtree_gen(tree: &mut Quadtree<u16>, n: usize) {
    tree.subdivide();
    for i in 0..4 {
        if (rand::random::<f32>() < WEIGHTS[n]) && n > 1 {
            quadtree_gen(&mut tree[i], n - 1);
        } else {
            tree[i] = Quadtree::Leaf(n as u16 - 1);
        }
    }
}

pub fn iter<'a, T: Clone>(tree: &'a Quadtree<T>, pos: u32, depth: u16) -> Vec<QuadtreeFlat<'a, T>> {
    match tree {
        Quadtree::Leaf(item) => vec![QuadtreeFlat::new(pos, item)],
        Quadtree::Stem(array) => {
            let mut items = Vec::new();
            for (index, item) in array.iter().enumerate() {
                let pos = pos | ((index as u32) << (2 * depth as u32));
                items.extend(iter(item, pos, depth + 1));
            }
            items
        }
    }
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct RoomTile {
    pub size: u8,
    pub tiles: Vec<String>,
    pub data: Vec<Vec<usize>>,
}
