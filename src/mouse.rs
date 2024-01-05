use std::ops::Sub;

use crate::utils::now;

pub(crate) struct MouseTracker {
    buttons: [Button; 3],
    current_pos: FloatPos,
}
impl MouseTracker {
    pub fn new() -> Self {
        Self {
            buttons: [Button::new(), Button::new(), Button::new()],
            current_pos: FloatPos::new(0.0, 0.0),
        }
    }
    pub fn is_down(&self, n: usize) -> bool {
        self.buttons[n].is_down()
    }
    pub fn set_down(&mut self, n: usize) {
        self.buttons[n].set_down(self.current_pos);
    }
    pub fn set_up(&mut self, n: usize) {
        self.buttons[n].set_up(self.current_pos);
    }
    pub fn get_time_held(&mut self, n: usize) -> Option<f64> {
        self.buttons[n].get_time_held()
    }
    pub fn set_current_pos(&mut self, pos: FloatPos) {
        self.current_pos = pos;
    }
    pub fn get_current_pos(&self) -> FloatPos {
        self.current_pos
    }
    pub fn get_pos(&self, n: usize) -> Option<FloatPos> {
        self.buttons[n].get_pos()
    }
}

struct Button {
    down: bool,
    timestamp: Option<f64>,
    pos: Option<FloatPos>,
}
impl Button {
    pub fn new() -> Self {
        Self {
            down: false,
            timestamp: None,
            pos: None,
        }
    }
    pub fn is_down(&self) -> bool {
        self.down
    }
    pub fn set_down(&mut self, pos: FloatPos) {
        self.down = true;
        self.timestamp = Some(now());
        self.pos = Some(pos);
    }
    pub fn set_up(&mut self, pos: FloatPos) {
        self.down = false;
        self.timestamp = Some(now());
        self.pos = Some(pos);
    }
    pub fn get_time_held(&self) -> Option<f64> {
        if self.timestamp == None {
            return None;
        }
        Some(now() - self.timestamp.unwrap())
    }
    pub fn get_pos(&self) -> Option<FloatPos> {
        self.pos
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FloatPos {
    pub x: f32,
    pub y: f32,
}
impl FloatPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
    pub fn max(&self) -> f32 {
        if self.x > self.y { self.x } else { self.y }
    }
}
impl Sub for FloatPos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
