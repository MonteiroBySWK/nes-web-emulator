import init, { Emulator } from './pkg/web_nes.js';

let emu = null;
let frameCount = 0;

// wasm-pack build --target web --out-dir web/pkg

async function loadWasm() {
    try {
        await init();
        return Emulator;
    } catch (e) {
        console.error("Failed to load WASM module:", e);
        throw e;
    }
}

async function loadROM(file, Emulator) {
    const arrayBuffer = await file.arrayBuffer();
    const romData = new Uint8Array(arrayBuffer);
    
    console.log("ROM size:", romData.length);
    
    try {
        emu = new Emulator("nes-screen", romData);
        requestAnimationFrame(gameLoop);
    } catch (e) {
        console.error("Error creating emulator:", e);
        updateDebug(`Error: ${e.message}`);
    }
}

function gameLoop() {
    updateDebug("Running...");
    try {
        emu.tick();
        // Atualiza os registradores na div de CPU
        // const regs = emu.get_registers();
        // document.getElementById("regA").textContent = regs.A;
        // document.getElementById("regX").textContent = regs.X;
        // document.getElementById("regY").textContent = regs.Y;
        // document.getElementById("regPC").textContent = regs.PC;
        // document.getElementById("regS").textContent = regs.S;
        // document.getElementById("regP").textContent = regs.P;
         frameCount++;
        updateDebug(`Frames: ${frameCount} - Canvas size: ${document.getElementById("nes-screen").width}x${document.getElementById("nes-screen").height}`);
    } catch (e) {
        console.error("Error in game loop:", e);
        updateDebug(`Error: ${e.message}`);
    }
    requestAnimationFrame(gameLoop);
}

function updateDebug(message) {
    document.getElementById('debug').textContent = message;
}

async function initialize() {
    try {
        const Emulator = await loadWasm();
        updateDebug("WASM loaded successfully. Please load a ROM.");
        
        const romInput = document.getElementById('rom-input');
        
        romInput.addEventListener('change', async (e) => {
            if (e.target.files.length > 0) {
                await loadROM(e.target.files[0], Emulator);
            }
            updateDebug("ROM loaded successfully. Press Start to play.");
        });
    } catch (e) {
        console.error("Initialization error:", e);
        updateDebug(`Init Error: ${e.message}`);
    }
}

initialize().catch(console.error);

