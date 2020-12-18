pub struct QueueFamilyIndices {
    pub graphics_family: i32,
    pub present_family: i32,
}

impl QueueFamilyIndices {
    pub fn new() -> Self {
        Self {
            graphics_family: -1,
            present_family: -1,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family >= 0 && self.present_family >= 0
    }
}
