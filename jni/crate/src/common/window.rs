pub trait WindowHints {
    fn set_control_position(&self, idx: usize, left: i32, top: i32, right: i32, bottom: i32);
    fn set_frame_size(&self, idx: usize, size: u32);
}
