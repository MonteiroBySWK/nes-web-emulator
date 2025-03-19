#[derive(Default, Debug, Clone, Copy)]
pub struct Controller {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Controller {
    pub fn new() -> Self {
        Self::default()
    }

    /// Atualiza o estado de um botão com base na entrada.
    /// `pressed` indica se a tecla foi pressionada (true) ou liberada (false).
    pub fn update(&mut self, key: Key, pressed: bool) {
        use Key::*;
        match key {
            A => self.a = pressed,
            B => self.b = pressed,
            Select => self.select = pressed,
            Start => self.start = pressed,
            Up => self.up = pressed,
            Down => self.down = pressed,
            Left => self.left = pressed,
            Right => self.right = pressed,
        }
    }
}

/// Enumeração dos botões do controle do NES.
#[derive(Debug)]
pub enum Key {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}