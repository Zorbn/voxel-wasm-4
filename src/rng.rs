pub struct Rng {
    state: u32,
}

impl Rng {
    pub const fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    // Get a random number using 32bit xorshift.
    pub fn range(&mut self, max: u32) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state % max
    }
}