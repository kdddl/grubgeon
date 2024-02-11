#[derive(Copy, Clone, Debug)]
pub enum Input {
    Up,
    Left,
    Right,
    Down,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
    Inventory,
    None,
    Quit,
    Number(char),
}

pub trait GetInput {
    fn get_input(&self) -> Input;
}
