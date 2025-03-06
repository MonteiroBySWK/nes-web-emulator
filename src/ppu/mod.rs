pub struct PPU {
    ctrl: u8, // $2000
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: [u8; 256],
    scroll: u8,  // Pode ser dividido em fine/coarse
    addr: u16,
    vram: [u8; 0x4000],

    // Registradores internos:
    v: u16,
    t: u16,
    x: u8,
    w: bool,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: [0; 256],
            scroll: 0,
            addr: 0,
            vram: [0; 0x4000],
            v: 0,
            t: 0,
            x: 0,
            w: false
        }
    }
}