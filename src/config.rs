pub struct Config {
    pub max_visible: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { max_visible: 10 }
    }
}
