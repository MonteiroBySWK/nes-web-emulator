// Tabelas auxiliares baseadas no hardware do NES (valores simplificados)
const LENGTH_TABLE: [u8; 16] = [10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14];
const NOISE_TIMER_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160,
    202, 254, 380, 508, 1016, 2034, 4068, 0,
];

/// APU – Audio Processing Unit
/// Agrupa os canais de áudio. Cada chamada de tick atualiza os canais.
pub struct APU {
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    triangle: TriangleChannel,
    noise: NoiseChannel,
    dmc: DMCChannel,
    status: u8,         // Status dos canais (registrador 0x4015)
    frame_counter: u16, // Conta os frames para os modos de frame sequencer
}

impl APU {
    pub fn new() -> Self {
        APU {
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DMCChannel::new(),
            status: 0,
            frame_counter: 0,
        }
    }

    /// Tick: atualiza cada canal (deve ser chamado em ciclos regulares)
    pub fn tick(&mut self) {
        self.pulse1.tick();
        self.pulse2.tick();
        self.triangle.tick();
        self.noise.tick();
        self.dmc.tick();
        self.frame_counter = self.frame_counter.wrapping_add(1);
        // Aqui poderíamos implementar o frame sequencer que gera clocks para os envelopes, sweeps e
        // contadores de length a cada 240 Hz (modo 4 ou 5 do frame counter)
    }

    /// write_register: Simula os writes na faixa 0x4000 a 0x4017 mapeados para a APU.
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => self.pulse1.write_control(value),
            0x4001 => self.pulse1.write_sweep(value),
            0x4002 => self.pulse1.write_timer_low(value),
            0x4003 => self.pulse1.write_timer_high(value),
            0x4004 => self.pulse2.write_control(value),
            0x4005 => self.pulse2.write_sweep(value),
            0x4006 => self.pulse2.write_timer_low(value),
            0x4007 => self.pulse2.write_timer_high(value),
            0x4008 => self.triangle.write_linear_counter(value),
            0x400A => self.triangle.write_timer_low(value),
            0x400B => self.triangle.write_timer_high(value),
            0x400C => self.noise.write_control(value),
            0x400E => self.noise.write_timer(value),
            0x400F => self.noise.write_length_counter(value),
            0x4010 => self.dmc.write_control(value),
            0x4011 => self.dmc.write_output_level(value),
            0x4012 => self.dmc.write_sample_address(value),
            0x4013 => self.dmc.write_sample_length(value),
            0x4015 => {
                self.status = value;
                self.pulse1.set_enabled(value & 0x01 != 0);
                self.pulse2.set_enabled(value & 0x02 != 0);
                self.triangle.set_enabled(value & 0x04 != 0);
                self.noise.set_enabled(value & 0x08 != 0);
                self.dmc.set_enabled(value & 0x10 != 0);
            },
            0x4017 => {
                // Frame counter mode e reset de sequenciador (não implementado totalmente)
            },
            _ => {}
        }
    }

    /// read_register: Apenas o registrador de status (0x4015) é lido.
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => self.status,
            _ => 0,
        }
    }
}

// =============================
// Pulse Channel
// =============================
pub struct PulseChannel {
    enabled: bool,
    volume: u8,         // 0-15 (volume ou envelope)
    duty_cycle: u8,     // 0-3: seleciona a forma de onda (usando uma tabela de duty cycle)
    timer: u16,         // Contador que determina o período da onda
    timer_reload: u16,  // Valor para recarregar o timer
    length_counter: u8, // Controla a duração do som
    envelope: u8,       // Valor corrente do envelope (decaimento do volume)
    constant_volume: bool,
    envelope_loop: bool,
    sweep: u8,          // Parâmetro de varredura (sweep) – modula a frequência
}

impl PulseChannel {
    pub fn new() -> Self {
        PulseChannel {
            enabled: false,
            volume: 15,
            duty_cycle: 0,
            timer: 0,
            timer_reload: 0,
            length_counter: 0,
            envelope: 15,
            constant_volume: false,
            envelope_loop: false,
            sweep: 0,
        }
    }

    /// tick: Atualiza o timer, avança a posição na duty sequencia e decai o envelope.
    pub fn tick(&mut self) {
        if self.enabled && self.length_counter > 0 {
            // Atualiza o timer – quando chega a zero,
            // recarrega e (idealmente) avança a posição na duty table.
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_reload;
                // Aqui seria a lógica para avançar a posição na duty table
                // e produzir o sinal PWM correspondente.
            }

            // Atualiza o envelope se não estiver em modo volume constante
            if !self.constant_volume {
                if self.envelope > 0 {
                    self.envelope -= 1;
                } else if self.envelope_loop {
                    self.envelope = self.volume;
                }
            }
        }
    }

    // Escrita em 0x4000 – Controle: duty, volume/envelope e flags de envelope loop/volume constante.
    pub fn write_control(&mut self, value: u8) {
        self.constant_volume = value & 0x10 != 0;
        self.volume = value & 0x0F;
        self.envelope_loop = value & 0x20 != 0;
        self.duty_cycle = (value >> 6) & 0x03;
    }

    // Escrita em 0x4001 – Sweep; neste exemplo, apenas armazena o valor.
    pub fn write_sweep(&mut self, value: u8) {
        self.sweep = value;
        // Em uma implementação completa, a varredura modificaria timer_reload periodicamente.
    }

    // Escrita em 0x4002 – Baixa parte do timer.
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | value as u16;
    }

    // Escrita em 0x4003 – Alta parte do timer e recarrega o length counter.
    pub fn write_timer_high(&mut self, value: u8) {
        self.timer_reload = (self.timer_reload & 0x00FF) | (((value & 0x07) as u16) << 8);
        self.length_counter = LENGTH_TABLE[(value & 0x0F) as usize % LENGTH_TABLE.len()];
    }
    
    // Ativa ou desativa o canal. Se desativado, zera o length counter.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
}

// =============================
// Triangle Channel
// =============================
pub struct TriangleChannel {
    enabled: bool,
    timer: u16,
    timer_reload: u16,
    sequence_position: u8, // Posiciona de 0 a 31 na sequência triangular
    length_counter: u8,
    linear_counter: u8,    // Valor decrescente que controla a duração da onda triangular
    linear_reload: u8,
    linear_control: bool,
}

impl TriangleChannel {
    pub fn new() -> Self {
        TriangleChannel {
            enabled: false,
            timer: 0,
            timer_reload: 0,
            sequence_position: 0,
            length_counter: 0,
            linear_counter: 0,
            linear_reload: 0,
            linear_control: false,
        }
    }
    
    /// tick: Atualiza o timer, avança a sequência triangular e decrementa o linear counter.
    pub fn tick(&mut self) {
        if self.enabled && self.length_counter > 0 && self.linear_counter > 0 {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_reload;
                self.sequence_position = (self.sequence_position + 1) % 32;
            }
            if self.linear_counter > 0 {
                self.linear_counter -= 1;
            } else if self.linear_control {
                self.linear_counter = self.linear_reload;
            }
        }
    }
    
    // Escrita em 0x4008 – Linear counter (parte alta)
    pub fn write_linear_counter(&mut self, value: u8) {
        self.linear_reload = value & 0x7F;
        self.linear_control = value & 0x80 != 0;
    }
    // Escrita em 0x400A – Baixa parte do timer
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | value as u16;
    }
    // Escrita em 0x400B – Alta parte do timer e atualiza o length counter
    pub fn write_timer_high(&mut self, value: u8) {
        self.timer_reload = (self.timer_reload & 0x00FF) | (((value & 0x07) as u16) << 8);
        self.length_counter = LENGTH_TABLE[((value >> 3) & 0x1F) as usize % LENGTH_TABLE.len()];
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
}

// =============================
// Noise Channel
// =============================
pub struct NoiseChannel {
    enabled: bool,
    timer: u16,
    timer_reload: u16,
    length_counter: u8,
    envelope: u8,
    constant_volume: bool,
    envelope_loop: bool,
    shift_register: u16, // Usado para gerar ruído (LFSR de 15 bits)
}

impl NoiseChannel {
    pub fn new() -> Self {
        NoiseChannel {
            enabled: false,
            timer: 0,
            timer_reload: 0,
            length_counter: 0,
            envelope: 15,
            constant_volume: false,
            envelope_loop: false,
            shift_register: 1, // Valor inicial não pode ser 0
        }
    }
    
    pub fn tick(&mut self) {
        if self.enabled && self.length_counter > 0 {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_reload;
                // Calcula o bit de feedback usando os dois bits menos significativos
                let feedback = ((self.shift_register & 1) ^ ((self.shift_register >> 1) & 1)) & 1;
                self.shift_register >>= 1;
                // Insere o feedback no bit 14 (para um LFSR de 15 bits)
                self.shift_register |= feedback << 14;
            }
            // Atualiza o envelope se não estiver em modo de volume constante
            if !self.constant_volume {
                if self.envelope > 0 {
                    self.envelope -= 1;
                } else if self.envelope_loop {
                    self.envelope = 15;
                }
            }
        }
    }
    
    // Escrita em 0x400C – Controle do canal Noise
    pub fn write_control(&mut self, value: u8) {
        self.constant_volume = value & 0x10 != 0;
        self.envelope_loop = value & 0x20 != 0;
        self.envelope = value & 0x0F;
    }
    // Escrita em 0x400E – Define o timer_reload usando a tabela de ruído
    pub fn write_timer(&mut self, value: u8) {
        self.timer_reload = NOISE_TIMER_TABLE[value as usize % NOISE_TIMER_TABLE.len()];
    }
    // Escrita em 0x400F – Atualiza o length counter
    pub fn write_length_counter(&mut self, value: u8) {
        self.length_counter = LENGTH_TABLE[value as usize % LENGTH_TABLE.len()];
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
}

// =============================
// DMC Channel (Delta Modulation Channel)
// =============================
pub struct DMCChannel {
    enabled: bool,
    timer: u16,
    timer_reload: u16,
    output_level: u8,        // 7 bits (0-127)
    sample_buffer: Option<u8>, // Buffer para o sample atual
    sample_address: u16,
    sample_length: u16,
    current_sample: u8,
}

impl DMCChannel {
    pub fn new() -> Self {
        DMCChannel {
            enabled: false,
            timer: 0,
            timer_reload: 0,
            output_level: 0,
            sample_buffer: None,
            sample_address: 0,
            sample_length: 0,
            current_sample: 0,
        }
    }
    
    pub fn tick(&mut self) {
        if self.enabled && self.sample_length > 0 {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.timer = self.timer_reload;
                // Em uma implementação mais completa, o DMC faria delta modulation:
                // - Ler um byte de sample da memória se o buffer estiver vazio.
                // - Ajustar output_level de acordo com o bit lido.
                if let Some(sample) = self.sample_buffer {
                    self.output_level = sample;
                    // Atualizar current_sample, aplicar delta, etc.
                }
            }
        }
    }
    
    // Escrita em 0x4010 – Controle do DMC (frequência e flags), simplificado aqui.
    pub fn write_control(&mut self, _value: u8) {
        // Implementar leitura de flags, taxa de clock etc.
    }
    // Escrita em 0x4011 – Define output level inicial
    pub fn write_output_level(&mut self, value: u8) {
        self.output_level = value & 0x7F;
    }
    // Escrita em 0x4012 – Define o endereço base para samples (0xC000 | (value << 6))
    pub fn write_sample_address(&mut self, value: u8) {
        self.sample_address = 0xC000 | ((value as u16) << 6);
    }
    // Escrita em 0x4013 – Define a quantidade de bytes do sample (length)
    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = ((value as u16) << 4) | 1;
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.sample_length = 0;
        }
    }
}