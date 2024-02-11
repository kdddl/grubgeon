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
