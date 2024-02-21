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
    MenuPrev,
    MenuNext,
    Select,
    EnterText,
}

pub enum TextInput {
    Char(char),
    Exit,
    Backspace,
    None,
}

pub trait GetInput {
    fn get_input(&self) -> Input;
    fn get_text_input(&self) -> TextInput;
}
