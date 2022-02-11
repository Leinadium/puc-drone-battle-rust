use std::time::Duration;

pub struct Field { }

impl Field {
    pub fn new() -> Field {
        Field {}
    }

    pub fn restart(&mut self) {

    }

    pub fn do_tick(&mut self, _duration: &Duration) {

    }
}

pub struct Path { }