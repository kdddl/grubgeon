#[derive(Debug, serde::Serialize, serde::Deserialize, Copy, Clone)]
pub struct Tile {
    pub r#char: char,
    pub fore: u8,
    pub back: u8,
    pub r#move: bool,
}

impl Tile {
    pub fn new(r#char: char, fore: u8, back: u8, r#move: bool) -> Self {
        Self {
            r#char,
            fore,
            back,
            r#move,
        }
    }

    pub fn from_string<S: Into<String>>(s: S, fore: Option<u8>, back: Option<u8>) -> Vec<Self> {
        Into::<String>::into(s)
            .chars()
            .map(|x| Tile::new(x, fore.unwrap_or(15), back.unwrap_or(0), false))
            .collect()
    }
}
