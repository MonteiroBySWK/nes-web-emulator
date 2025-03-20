use super::PPU;

impl PPU {
    // Frame timing related methods
    pub(crate) fn handle_frame_timing(&mut self) -> bool {
        // Skip cycle on odd frames when rendering is enabled
        if self.rendering_enabled && self.odd_frame && self.scanline == 261 && self.cycle == 339 {
            self.cycle = 0;
            self.scanline = 0;
            self.odd_frame = !self.odd_frame;
            self.frame += 1;
            return true;
        }
        false
    }
}
