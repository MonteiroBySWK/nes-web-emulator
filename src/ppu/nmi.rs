use super::PPU;

impl PPU {
    pub(crate) fn check_nmi(&mut self) -> bool {
        // Check and generate NMI if conditions are met
        if self.scanline == 241 && self.cycle == 1 {
            self.status |= 0x80; // Set VBlank flag
            self.nmi_occurred = true;
            return self.nmi_output;
        }
        false
    }

    pub(crate) fn clear_nmi(&mut self) {
        if self.scanline == 261 && self.cycle == 1 {
            self.status &= !0x80; // Clear VBlank flag
            self.nmi_occurred = false;
        }
    }
}
