import init, { Emulator } from './pkg/web_nes.js';

let emu = null;
let frameCount = 0;
let isRunning = false;

// Keyboard mapping
const keyMap = {
    'z': 'A',
    'x': 'B',
    'Control': 'Select',
    'Enter': 'Start',
    'ArrowUp': 'Up',
    'ArrowDown': 'Down',
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right'
};

async function loadWasm() {
    try {
        await init();
        updateDebug("WASM loaded successfully");
    } catch (e) {
        console.error("Failed to load WASM:", e);
        updateDebug(`WASM Error: ${e.message}`);
        throw e;
    }
}

async function loadROM(file) {
    try {
        const arrayBuffer = await file.arrayBuffer();
        const romData = new Uint8Array(arrayBuffer);
        
        // More detailed logging
        const romInfo = {
            size: romData.length,
            header: Array.from(romData.slice(0, 16)).map(b => b.toString(16).padStart(2, '0')),
            prgSize: romData[4] * 16384,
            chrSize: romData[5] * 8192,
            mapper: (romData[6] >> 4) | (romData[7] & 0xF0),
            flags: {
                verticalMirroring: !!(romData[6] & 0x01),
                batteryBacked: !!(romData[6] & 0x02),
                hasTrainer: !!(romData[6] & 0x04),
                fourScreen: !!(romData[6] & 0x08)
            }
        };
        console.log("ROM details:", romInfo);
        
        // Validate ROM before creating emulator
        if (romData.length < 16 || 
            romData[0] !== 0x4E || // 'N'
            romData[1] !== 0x45 || // 'E'
            romData[2] !== 0x53 || // 'S'
            romData[3] !== 0x1A) {
            throw new Error("Invalid iNES header");
        }

        // Create new emulator instance with try-catch
        console.log("Creating emulator instance...");
        try {
            emu = new Emulator("nes-screen", romData);
            console.log("Emulator instance created successfully:", emu);
        } catch (e) {
            console.error("Failed to create emulator:", e);
            throw e;
        }

        isRunning = true;
        requestAnimationFrame(gameLoop);
        updateDebug("ROM loaded successfully");
        setupControls();
        
    } catch (e) {
        console.error("Error loading ROM:", e);
        updateDebug(`ROM Error: ${e.message}`);
        isRunning = false;
        emu = null;
    }
}

function gameLoop() {
    if (!isRunning || !emu) return;

    try {
        // Execute one frame
        emu.tick();
        
        // Update debug info
        updateRegisters();
        frameCount++;
        
        // Update status
        const canvas = document.getElementById("nes-screen");
        updateDebug(
            `Frame: ${frameCount} | ` +
            `Canvas: ${canvas.width}x${canvas.height}`
        );
        
        // Schedule next frame
        requestAnimationFrame(gameLoop);
    } catch (e) {
        console.error("Error in game loop:", e);
        updateDebug(`Runtime Error: ${e.message}`);
        isRunning = false;
    }
}

function updateRegisters() {
    updateRegistersCPU();
    updateRegistersPPU();
}

function updateRegistersCPU() {
    const regs = emu.get_registers_cpu();
    for (const reg of ['A', 'X', 'Y', 'PC', 'S', 'P']) {
        const elem = document.getElementById(`reg${reg}`);
        if (elem) {
            elem.textContent = regs[reg].toString(16).toUpperCase().padStart(2, '0');
        }
    }
}

function updateRegistersPPU() {
    const regs = emu.get_registers_ppu();
    const registerMap = {
        'PPUCTRL': 'CTRL',
        'PPUMASK': 'MASK',
        'PPUSTATUS': 'STATUS',
        'OAMADDR': 'OAMADDR',
        'PPUSCROLL': 'SCROLL',
        'PPUADDR': 'ADDR',
        'PPUADDRFULL': 'ADDRFULL'
    };

    for (const [htmlId, regKey] of Object.entries(registerMap)) {
        const elem = document.getElementById(`reg${htmlId}`);
        if (elem && regs[regKey] !== undefined) {
            // Use padStart(4) for ADDRFULL since it's a 16-bit value
            const padLength = regKey === 'ADDRFULL' ? 4 : 2;
            elem.textContent = regs[regKey].toString(16).toUpperCase().padStart(padLength, '0');
        }
    }
}

function setupControls() {
    // Remove any existing listeners
    document.removeEventListener('keydown', handleKeyDown);
    document.removeEventListener('keyup', handleKeyUp);
    
    // Add new listeners
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);
}

function handleKeyDown(event) {
    const nesKey = keyMap[event.key];
    if (nesKey && emu) {
        event.preventDefault();
        emu.key_down(nesKey); // Usar o valor mapeado, não a tecla original
    }
}

function handleKeyUp(event) {
    const nesKey = keyMap[event.key];
    if (nesKey && emu) {
        event.preventDefault();
        emu.key_up(nesKey); // Usar o valor mapeado, não a tecla original
    }
}

function updateDebug(message) {
    const debug = document.getElementById('debug');
    if (debug) {
        debug.textContent = message;
    }
}

async function initialize() {
    try {
        await loadWasm();
        
        const romInput = document.getElementById('rom-input');
        if (romInput) {
            romInput.addEventListener('change', async (e) => {
                if (e.target.files.length > 0) {
                    await loadROM(e.target.files[0]);
                }
            });
        }
        
        updateDebug("Ready - Please load a ROM");
    } catch (e) {
        console.error("Initialization error:", e);
        updateDebug(`Init Error: ${e.message}`);
    }
}

// Start initialization
initialize().catch(console.error);

