mod cpu;
mod rom;
mod input;
mod ppu;
mod apu;
mod mapper;
mod bus;

use crate::{ cpu::CPU, ppu::PPU, rom::ROM, bus::BUS , apu::APU};
use wasm_bindgen::prelude::*;
use web_sys::{ CanvasRenderingContext2d, HtmlCanvasElement };

#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, rom_data: &[u8]) -> Result<Emulator, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas.get_context("2d")?.unwrap().dyn_into::<CanvasRenderingContext2d>()?;

        let mut ppu = PPU::new();
        let rom = ROM::from_bytes(rom_data).map_err(|e|
            JsValue::from_str(&format!("ROM error: {}", e))
        )?;
        let apu = APU::new();

        let bus = BUS::new(ppu, rom, apu);
        let mut cpu = CPU::new(bus);
        cpu.reset();

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
        // Obtém o framebuffer (array de pixels em RGB)
        let framebuffer = self.cpu.bus.ppu.get_framebuffer();

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
        // Assumindo que 'cpu.registers' possui os campos: a, x, y, pc, s, p
        js_sys::Reflect::set(&regs, &JsValue::from_str("A"), &JsValue::from_f64(self.cpu.registers.acc as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("X"), &JsValue::from_f64(self.cpu.registers.index_x as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("Y"), &JsValue::from_f64(self.cpu.registers.index_y as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("PC"), &JsValue::from_f64(self.cpu.registers.program_counter as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("S"), &JsValue::from_f64(self.cpu.registers.stack_pointer as f64)).unwrap();
        js_sys::Reflect::set(&regs, &JsValue::from_str("P"), &JsValue::from_f64(self.cpu.registers.status_register as f64)).unwrap();
        regs.into()
    }
}
