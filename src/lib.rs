mod cpu;
mod rom;
mod input;
mod ppu;
mod apu;
mod mapper;
mod bus;
mod nes;

use crate::mapper::Mapper0;
use crate::{ cpu::CPU, ppu::PPU, rom::{ROM,Mirroring}, bus::BUS , apu::APU};
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
        web_sys::console::log_1(&format!(
            "Creating Emulator - ROM size: {}, First bytes: {:?}",
            rom_data.len(),
            &rom_data.get(0..16).unwrap_or(&[]).iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
        ).into());

        // Get canvas and context with error handling
        let window = web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window found"))?;
        let document = window.document()
            .ok_or_else(|| JsValue::from_str("No document found"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Initialize components with detailed error handling
        let mut ppu = PPU::new();

        web_sys::console::log_1(&"Creating ROM...".into());
        let rom = ROM::from_bytes(rom_data)
            .map_err(|e| {
                web_sys::console::error_1(&format!("ROM creation failed: {:?}", e).into());
                JsValue::from_str(&format!("ROM error: {}", e))
            })?;
    

    
        // Configure PPU mirroring from ROM
        let mirroring = rom.get_mirroring();
        ppu.set_mirroring(mirroring);
        web_sys::console::log_1(&format!("PPU Mirroring set to: {:?}", mirroring).into());

        let chr_data = rom.mapper.get_chr_rom(); // Você precisará implementar este método em sua struct ROM
        web_sys::console::log_1(&format!("CHR data size: {}", chr_data.len()).into());

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
            self.render();
        }
    }

    fn render(&self) {
        web_sys::console::log_1(&format!("PPU Mask: {:02x}", self.cpu.bus.ppu.mask).into());
        web_sys::console::log_1(&format!("PPU Ctrl: {:02x}", self.cpu.bus.ppu.ctrl).into());
        // Obtém o framebuffer (array de pixels em RGB)
        let framebuffer = self.cpu.bus.ppu.get_framebuffer();
        web_sys::console::log_1(&format!("Framebuffer (primeiros 10 bytes): {:?}", &framebuffer[0..30]).into());
        // Converte cada pixel (RGB de 3 bytes) para RGBA (4 bytes, alfa=255)
        let mut rgba_buffer = Vec::with_capacity(256 * 240 * 4);
        for pixel in framebuffer.chunks(3) {
            rgba_buffer.push(pixel[0]);
            rgba_buffer.push(pixel[1]);
            rgba_buffer.push(pixel[2]);
            rgba_buffer.push(255); // Alfa fixo em 255 (opaco)
        }

        // Cria um objeto ImageData usando o buffer RGBA e o tamanho do canvas
        match
            web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(&rgba_buffer),
                256,
                240
            )
        {
            Ok(image_data) => {
                if let Err(e) = self.context.put_image_data(&image_data, 0.0, 0.0) {
                    web_sys::console::error_1(
                        &format!("Erro ao colocar dados de imagem: {:?}", e).into()
                    );
                }
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Erro ao criar dados de imagem: {:?}", e).into()
                );
            }
        }
    }

    #[wasm_bindgen]
    pub fn get_registers(&self) -> JsValue {
        let regs = js_sys::Object::new();
        // Assumindo que 'cpu.registers' possui os campos: acc, index_x, index_y, program_counter, stack_pointer, status_register
        js_sys::Reflect::set(&regs, &JsValue::from_str("A"), &JsValue::from_f64(self.cpu.registers.acc as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("X"), &JsValue::from_f64(self.cpu.registers.index_x as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("Y"), &JsValue::from_f64(self.cpu.registers.index_y as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("PC"), &JsValue::from_f64(self.cpu.registers.program_counter as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("S"), &JsValue::from_f64(self.cpu.registers.stack_pointer as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("P"), &JsValue::from_f64(self.cpu.registers.status_register as f64)).unwrap();
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