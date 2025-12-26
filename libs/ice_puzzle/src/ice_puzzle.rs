use platform_types::Dir;
use xs::Xs;

#[derive(Clone, Debug)]
pub struct State {
    count: u8, // Temp to just have something easy but visible
}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
        Self {
            count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.count = self.count.saturating_add(1);
    }

    pub fn is_complete(&self) -> bool {
        self.count == u8::MAX
    }

    pub fn r#move(&mut self, dir: Dir) {
        self.count = u8::MAX;
    }
}