#[derive(Default)]
pub struct Score {
    pub max_points: u32,
    pub received_points: u32,
    pub test_output: String,
    pub clippy_output: String,
}
