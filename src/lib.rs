mod cpu;
mod rom;
mod input;
mod ppu;
mod apu;
mod mapper;
mod bus;
mod nes;

use crate::{ cpu::CPU, ppu::PPU, rom::ROM, bus::BUS, apu::APU };
use wasm_bindgen::prelude::*;
use web_sys::{ CanvasRenderingContext2d, HtmlCanvasElement };
use crate::input::Key;

#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, rom_data: &[u8]) -> Result<Emulator, JsValue> {
        // Log ROM data size and first few bytes
        web_sys::console::log_1(
            &format!(
                "Creating Emulator - ROM size: {}, First bytes: {:?}",
                rom_data.len(),
                &rom_data
                    .get(0..16)
                    .unwrap_or(&[])
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
            ).into()
        );

        // Get canvas and context with error handling
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("No document found"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        // Set canvas size and scaling
        canvas.set_width(256);
        canvas.set_height(240);
        
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Disable image smoothing for sharp pixels
        context.set_image_smoothing_enabled(false);

        // Initialize components with detailed error handling
        let mut ppu = PPU::new();

        web_sys::console::log_1(&"Creating ROM...".into());
        let rom = ROM::from_bytes(rom_data).map_err(|e| {
            web_sys::console::error_1(&format!("ROM creation failed: {:?}", e).into());
            JsValue::from_str(&format!("ROM error: {}", e))
        })?;

        // Configure PPU mirroring from ROM
        let mirroring = rom.get_mirroring();
        ppu.set_mirroring(mirroring);
        web_sys::console::log_1(&format!("PPU Mirroring set to: {:?}", mirroring).into());

        let chr_data = rom.mapper.get_chr_rom(); // Você precisará implementar este método em sua struct ROM
        web_sys::console::log_1(&format!("CHR data size: {}", chr_data.len()).into());


        web_sys::console::log_1(&format!("CHR: {:?}", chr_data).into());
        ppu.load_chr_data(&chr_data);
        web_sys::console::log_1(&"CHR data loaded into PPU".into());

        web_sys::console::log_1(&"Creating APU...".into());
        let apu = APU::new();

        web_sys::console::log_1(&"Creating BUS...".into());
        let mut bus = BUS::new(ppu, rom, apu);

        bus.ppu.set_mirroring(bus.rom.get_mirroring());

        web_sys::console::log_1(&"Finish BUS...".into());

        web_sys::console::log_1(&"Creating CPU...".into());
        let mut cpu = CPU::new(bus);

        web_sys::console::log_1(&"Resetting CPU...".into());
        cpu.reset();

        web_sys::console::log_1(&"Emulator creation completed".into());
        Ok(Emulator {
            cpu,
            context,
        })
    }

    pub fn tick(&mut self) {
        let frame_complete = self.cpu.clock();

        if frame_complete {
            self.render().unwrap_or_else(|e| {
                web_sys::console::error_1(&format!("Render error: {:?}", e).into());
            });
        }
    }

    fn render(&self) -> Result<(), JsValue> {
        let framebuffer = self.cpu.bus.ppu.get_framebuffer();
        
        web_sys::console::log_1(&format!("Rendering frame, buffer size: {}", framebuffer.len()).into());
        
        let mut rgba_buffer = vec![0; 256 * 240 * 4];
        
        for i in 0..(256 * 240) {
            let src = i * 3;
            let dst = i * 4;
            if src + 2 < framebuffer.len() && dst + 3 < rgba_buffer.len() {
                rgba_buffer[dst] = framebuffer[src];
                rgba_buffer[dst + 1] = framebuffer[src + 1];
                rgba_buffer[dst + 2] = framebuffer[src + 2];
                rgba_buffer[dst + 3] = 255;
            }
        }

        let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba_buffer),
            256,
            240
        )?;

        self.context.clear_rect(0.0, 0.0, 256.0, 240.0);
        self.context.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_registers_cpu(&self) -> JsValue {
        let regs = js_sys::Object::new();

        let (acc, index_x, index_y, program_counter, stack_pointer, status_register) =
            self.cpu.get_all_registers();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("A"), &JsValue::from_f64(acc as f64))
            .unwrap();
        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("X"), &JsValue::from_f64(index_x as f64))
            .unwrap();
        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("Y"), &JsValue::from_f64(index_y as f64))
            .unwrap();
        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("PC"), &JsValue::from_f64(program_counter as f64))
            .unwrap();
        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("S"), &JsValue::from_f64(stack_pointer as f64))
            .unwrap();
        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("P"), &JsValue::from_f64(status_register as f64))
            .unwrap();
        regs.into()
    }

    #[wasm_bindgen]
    pub fn get_registers_ppu(&self) -> JsValue {
        let regs = js_sys::Object::new();

        let (ctrl, mask, status, oam_addr, scroll, addr_low, addr_full) =
            self.cpu.bus.ppu.get_all_registers();

        // PPU Registers
        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("CTRL"), &JsValue::from_f64(ctrl as f64))
            .unwrap();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("MASK"), &JsValue::from_f64(mask as f64))
            .unwrap();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("STATUS"), &JsValue::from_f64(status as f64))
            .unwrap();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("OAMADDR"), &JsValue::from_f64(oam_addr as f64))
            .unwrap();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("SCROLL"), &JsValue::from_f64(scroll as f64))
            .unwrap();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("ADDR"), &JsValue::from_f64(addr_low as f64))
            .unwrap();

        js_sys::Reflect
            ::set(&regs, &JsValue::from_str("ADDRFULL"), &JsValue::from_f64(addr_full as f64))
            .unwrap();

        regs.into()
    }

    #[wasm_bindgen]
    pub fn key_down(&mut self, key: &str) {
        if let Some(key) = map_key(key) {
            self.cpu.bus.controller.update(key, true);
        }
    }

    #[wasm_bindgen]
    pub fn key_up(&mut self, key: &str) {
        if let Some(key) = map_key(key) {
            self.cpu.bus.controller.update(key, false);
        }
    }
}

fn map_key(key: &str) -> Option<Key> {
    match key {
        "z" => Some(Key::A),
        "x" => Some(Key::B),
        "Control" => Some(Key::Select),
        "Enter" => Some(Key::Start),
        "ArrowUp" => Some(Key::Up),
        "ArrowDown" => Some(Key::Down),
        "ArrowLeft" => Some(Key::Left),
        "ArrowRight" => Some(Key::Right),
        _ => None,
    }
}
