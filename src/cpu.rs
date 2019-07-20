use crate::cpu_bus;
pub struct Cpu {
    pub a: i8,   // accumlator register
    pub x: i8,   // index register
    pub y: i8,   // index register
    pub sp: u16, // stack pointer       (Begin from 0x1FD) Upper Bit is fixed to 0x01
    pub pc: u16, // program counter
    pub p: Status,
    bus: cpu_bus::CpuBus,
}

pub struct Status {
    pub negative: bool,
    pub overflow: bool,
    pub reserved: bool,
    pub break_mode: bool,
    pub decimal: bool,
    pub interrupt: bool,
    pub zero: bool,
    pub carry: bool,
}

#[rustfmt::skip]
const CYCLE: [u8; 256] = [
     /*0x00*/ 7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
     /*0x10*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
     /*0x20*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
     /*0x30*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
     /*0x40*/ 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
     /*0x50*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
     /*0x60*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
     /*0x70*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 6, 7,
     /*0x80*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
     /*0x90*/ 2, 6, 2, 6, 4, 4, 4, 4, 2, 4, 2, 5, 5, 4, 5, 5,
     /*0xA0*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
     /*0xB0*/ 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
     /*0xC0*/ 2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
     /*0xD0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
     /*0xE0*/ 2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
     /*0xF0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];

enum Instruction {
    // A: Accumlator M: fetched memory data C: The flag to set by an instruction.
    ADC, // Add M to A with C: A += M + C
    SBC, // Substract M from A with C: A -= M - not C
    AND, // "AND" M with A: A &= M
    ORA, // "OR" M with A: A |= M
    EOR, // "XOR" M with A: A ^= M
    ASL, // Arithmetic shift left one bit: C = bit 7 of A
    LSR, // Logical shift right one bit: C = bit 0 of A
    ROL, // Rotate left one bit: C = bit 7 of A
    ROR, // Rotate right one bit: C = bit 0 of A
    BCC, // Branch on C clear
    BCS, // Branch on C set
    BEQ, // Branch on Z set (result equal)
    BNE, // Branch on Z clear (result not equal)
    BVC, // Branch on V clear
    BVS, // Branch on V set
    BPL, // Branch on N clear (result plus)
    BMI, // Branch on N set ( result minus)
    BIT, // Test Bits in M with A: N = bit 7 of M, V = bit 6 of M
    JMP, // Jump to new location: PC = ADDR
    JSR, // Jump to new location saving return address: PC = ADDR
    RTS, // Return from Subroutine
    BRK, // Force Break
    RTI, // Return from Interrupt)
    CMP, // Compare M and A
    CPX, // Compare M and X
    CPY, // Compare M and Y
    INC, // Increment M by one
    DEC, // Decrement M by one
    INX, // Increment X by one
    DEX, // Decrement X by one
    INY, // Increment Y by one
    DEY, // Decrement Y by one
    CLC, // Clear C flag)
    SEC, // Set C flag)
    CLI, // Clear Interrupt disable
    SEI, // Set Interrupt disable
    CLD, // Clear Decimal mode
    SED, // Set Decimal mode
    CLV, // Clear V flag
    LDA, // Load A from M
    LDX, // Load X from M
    LDY, // Load Y from M
    STA, // Store A to M
    STX, // Store X to M
    STY, // Store Y to M
    TAX, // Transfer A to X
    TXA, // Transfer X to A
    TAY, // Transfer A to Y
    TYA, // Transfer Y to A
    TSX, // Transfer S to X
    TXS, // Transfer X to S
    PHA, // Push A on stack
    PLA, // Pull A from stack
    PHP, // Push P on stack
    PLP, // Pull P from stack
    NOP, // No operation
    NOPI,
    NOPD,
    Unknown,
}

enum Addressing {
    Accumlator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Implied,
    Relative,
    IndirectX,
    IndirectY,
    Indirect,
    Unknown,
}

enum ReadSize {
    Word,
    Byte,
}

enum ReadResult {
    Data(u8),
    Addr(u16),
    None
}

impl Cpu {
    pub fn init(cpu_bus: cpu_bus::CpuBus) -> Cpu {
        Cpu {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x01FD,
            pc: 0x0000,
            p: Status {
                negative: false,
                overflow: false,
                reserved: true,
                break_mode: true,
                decimal: false,
                interrupt: true,
                zero: false,
                carry: false,
            },
            bus: cpu_bus,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;
        self.sp = 0x01FD;
        self.pc = 0x0000;
        self.p = Status {
            negative: false,
            overflow: false,
            reserved: true,
            break_mode: true,
            decimal: false,
            interrupt: true,
            zero: false,
            carry: false,
        };
        self.pc = if let ReadResult::Addr(i) = self.read(0xFFFC, ReadSize::Word) {
            i
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        };
    }

    fn read(&mut self, addr: u16, size: ReadSize) -> ReadResult {
        let bus = &self.bus;
        match size {
            ReadSize::Word => {
                let lower = bus.read_by_cpu(addr);
                let upper = bus.read_by_cpu(addr + 0x0001);
                let mut bit = (upper as u16) << 8;
                bit |= lower as u16;
                //println!("{} {} ", lower, upper);
                ReadResult::Addr(bit)
            }
            ReadSize::Byte => ReadResult::Data(bus.read_by_cpu(addr)),
        }
    }

    // fetch opcode (8-bit)
    fn fetch(&mut self) -> u8 {
        if let ReadResult::Data(i) = self.read(self.pc, ReadSize::Byte) {
            self.pc += if self.pc < 0xFFFF { 1 } else { 0 };
            //println!("{:x}", self.pc);
            i
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    fn fetch_opeland(&mut self, op_info: (Instruction, Addressing, u8)) -> ReadResult {
        match op_info.1 {
            Addressing::Accumlator => {ReadResult::None},
            Addressing::Immediate => {ReadResult::Data(self.fetch())},
            Addressing::Absolute => {
                let lower_bit = self.fetch();
                let upper_bit = self.fetch();
                let mut bit = (upper_bit as u16) << 8;
                bit |= lower_bit as u16;
                ReadResult::Addr(bit)
            },
            Addressing::ZeroPage => {ReadResult::Addr(self.fetch() as u16)},
            Addressing::ZeroPageX => {
                ReadResult::Addr((self.fetch() as u16 + self.x as u16) & 0xFF)
            },
            Addressing::ZeroPageY => {
                ReadResult::Addr(self.fetch() as u16 + self.y as u16 & 0xFF)
            },
            Addressing::AbsoluteX => {
                let lower_bit = self.fetch();
                let upper_bit = self.fetch();
                let mut bit = (upper_bit as u16) << 8;
                bit |= lower_bit as u16;
                ReadResult::Addr(bit + self.x as u16)
            },
            Addressing::AbsoluteY => {
                let lower_bit = self.fetch();
                let upper_bit = self.fetch();
                let mut bit = (upper_bit as u16) << 8;
                bit |= lower_bit as u16;
                ReadResult::Addr(bit + self.y as u16)
            },
            Addressing::Implied => {ReadResult::None},
            Addressing::Relative => {
                let addr = self.pc;
                let offset = self.fetch() as u16;
                ReadResult::Addr(addr + offset)
            },
            Addressing::Indirect => {
                let lower = self.fetch() as u16;
                let upper = self.fetch() as u16;
                ReadResult::Addr(upper + lower)
            },
            Addressing::IndirectX => {
                let mut bit = self.fetch() as u16;
                bit += self.x as u16;
                bit &= 0x00FF;
                ReadResult::Addr(bit)
            },
            Addressing::IndirectY => {
                let mut bit = self.fetch() as u16;
                bit += self.y as u16;
                bit &= 0x00FF;
                ReadResult::Addr(bit)
            },
            Addressing::Unknown => {
                println!("Unknown Addressing mode");
                ReadResult::None
            }
        }
    }

    fn exec(&self) {}

    pub fn run(&mut self) {
        let opcode = self.fetch();
        if self.pc < 0x8080 {
            let op_info = self.get_instruction_info(opcode);
            println!(
                "{:x} {:x} {}",
                opcode,
                op_info.2,
                if op_info.2 == 0 { "unknown" } else { "" }
            );
        }
    }

    fn get_instruction_info(&self, opcode: u8) -> (Instruction, Addressing, u8) {
        let index = opcode as usize;
        match opcode {
            //ADC
            0x69 => (Instruction::ADC, Addressing::Immediate, CYCLE[index]),
            0x65 => (Instruction::ADC, Addressing::ZeroPage, CYCLE[index]),
            0x75 => (Instruction::ADC, Addressing::ZeroPageX, CYCLE[index]),
            0x6D => (Instruction::ADC, Addressing::Absolute, CYCLE[index]),
            0x7D => (Instruction::ADC, Addressing::AbsoluteX, CYCLE[index]),
            0x79 => (Instruction::ADC, Addressing::AbsoluteY, CYCLE[index]),
            0x61 => (Instruction::ADC, Addressing::IndirectX, CYCLE[index]),
            0x71 => (Instruction::ADC, Addressing::IndirectY, CYCLE[index]),
            //SBC
            0xE9 => (Instruction::SBC, Addressing::Immediate, CYCLE[index]),
            0xE5 => (Instruction::SBC, Addressing::ZeroPage, CYCLE[index]),
            0xF5 => (Instruction::SBC, Addressing::ZeroPageX, CYCLE[index]),
            0xED => (Instruction::SBC, Addressing::Absolute, CYCLE[index]),
            0xFD => (Instruction::SBC, Addressing::AbsoluteX, CYCLE[index]),
            0xF9 => (Instruction::SBC, Addressing::AbsoluteY, CYCLE[index]),
            0xE1 => (Instruction::SBC, Addressing::IndirectX, CYCLE[index]),
            0xF1 => (Instruction::SBC, Addressing::IndirectY, CYCLE[index]),
            //AND
            0x29 => (Instruction::AND, Addressing::Immediate, CYCLE[index]),
            0x25 => (Instruction::AND, Addressing::ZeroPage, CYCLE[index]),
            0x35 => (Instruction::AND, Addressing::ZeroPageX, CYCLE[index]),
            0x2D => (Instruction::AND, Addressing::Absolute, CYCLE[index]),
            0x3D => (Instruction::AND, Addressing::AbsoluteX, CYCLE[index]),
            0x39 => (Instruction::AND, Addressing::AbsoluteY, CYCLE[index]),
            0x21 => (Instruction::AND, Addressing::IndirectX, CYCLE[index]),
            0x31 => (Instruction::AND, Addressing::IndirectY, CYCLE[index]),
            //ORA
            0x09 => (Instruction::ORA, Addressing::Immediate, CYCLE[index]),
            0x05 => (Instruction::ORA, Addressing::ZeroPage, CYCLE[index]),
            0x15 => (Instruction::ORA, Addressing::ZeroPageX, CYCLE[index]),
            0x0D => (Instruction::ORA, Addressing::Absolute, CYCLE[index]),
            0x1D => (Instruction::ORA, Addressing::AbsoluteX, CYCLE[index]),
            0x19 => (Instruction::ORA, Addressing::AbsoluteY, CYCLE[index]),
            0x01 => (Instruction::ORA, Addressing::IndirectX, CYCLE[index]),
            0x11 => (Instruction::ORA, Addressing::IndirectY, CYCLE[index]),
            //EOR
            0x49 => (Instruction::EOR, Addressing::Immediate, CYCLE[index]),
            0x45 => (Instruction::EOR, Addressing::ZeroPage, CYCLE[index]),
            0x55 => (Instruction::EOR, Addressing::ZeroPageX, CYCLE[index]),
            0x4D => (Instruction::EOR, Addressing::Absolute, CYCLE[index]),
            0x5D => (Instruction::EOR, Addressing::AbsoluteX, CYCLE[index]),
            0x59 => (Instruction::EOR, Addressing::AbsoluteY, CYCLE[index]),
            0x41 => (Instruction::EOR, Addressing::IndirectX, CYCLE[index]),
            0x51 => (Instruction::EOR, Addressing::IndirectY, CYCLE[index]),
            //ASL
            0x0A => (Instruction::ASL, Addressing::Accumlator, CYCLE[index]),
            0x06 => (Instruction::ASL, Addressing::ZeroPage, CYCLE[index]),
            0x16 => (Instruction::ASL, Addressing::ZeroPageX, CYCLE[index]),
            0x0E => (Instruction::ASL, Addressing::Absolute, CYCLE[index]),
            0x1E => (Instruction::ASL, Addressing::AbsoluteX, CYCLE[index]),
            //LSR
            0x4A => (Instruction::LSR, Addressing::Accumlator, CYCLE[index]),
            0x46 => (Instruction::LSR, Addressing::ZeroPage, CYCLE[index]),
            0x56 => (Instruction::LSR, Addressing::ZeroPageX, CYCLE[index]),
            0x4E => (Instruction::LSR, Addressing::Absolute, CYCLE[index]),
            0x5E => (Instruction::LSR, Addressing::AbsoluteX, CYCLE[index]),
            //ROL
            0x2A => (Instruction::ROL, Addressing::Accumlator, CYCLE[index]),
            0x26 => (Instruction::ROL, Addressing::ZeroPage, CYCLE[index]),
            0x36 => (Instruction::ROL, Addressing::ZeroPageX, CYCLE[index]),
            0x2E => (Instruction::ROL, Addressing::Absolute, CYCLE[index]),
            0x3E => (Instruction::ROL, Addressing::AbsoluteX, CYCLE[index]),
            //ROR
            0x6A => (Instruction::ROR, Addressing::Accumlator, CYCLE[index]),
            0x66 => (Instruction::ROR, Addressing::ZeroPage, CYCLE[index]),
            0x76 => (Instruction::ROR, Addressing::ZeroPageX, CYCLE[index]),
            0x6E => (Instruction::ROR, Addressing::Absolute, CYCLE[index]),
            0x7E => (Instruction::ROR, Addressing::AbsoluteX, CYCLE[index]),
            //BCC
            0x90 => (Instruction::BCC, Addressing::Relative, CYCLE[index]),
            //BCS
            0xB0 => (Instruction::BCS, Addressing::Relative, CYCLE[index]),
            //BEQ
            0xF0 => (Instruction::BEQ, Addressing::Relative, CYCLE[index]),
            //BNE
            0xD0 => (Instruction::BNE, Addressing::Relative, CYCLE[index]),
            //BVC
            0x50 => (Instruction::BVC, Addressing::Relative, CYCLE[index]),
            //BVS
            0x70 => (Instruction::BVS, Addressing::Relative, CYCLE[index]),
            //BPL
            0x10 => (Instruction::BPL, Addressing::Relative, CYCLE[index]),
            //BMI
            0x30 => (Instruction::BMI, Addressing::Relative, CYCLE[index]),
            //BIT
            0x24 => (Instruction::BIT, Addressing::ZeroPage, CYCLE[index]),
            0x2C => (Instruction::BIT, Addressing::Absolute, CYCLE[index]),
            //JMP
            0x4C => (Instruction::JMP, Addressing::Absolute, CYCLE[index]),
            0x6C => (Instruction::JMP, Addressing::Indirect, CYCLE[index]),
            //JSR
            0x20 => (Instruction::JSR, Addressing::Absolute, CYCLE[index]),
            //RTS
            0x60 => (Instruction::RTS, Addressing::Implied, CYCLE[index]),
            //BRK
            0x00 => (Instruction::BRK, Addressing::Implied, CYCLE[index]),
            //RTI
            0x40 => (Instruction::RTI, Addressing::Implied, CYCLE[index]),
            //CMP
            0xC9 => (Instruction::CMP, Addressing::Immediate, CYCLE[index]),
            0xC5 => (Instruction::CMP, Addressing::ZeroPage, CYCLE[index]),
            0xD5 => (Instruction::CMP, Addressing::ZeroPageX, CYCLE[index]),
            0xCD => (Instruction::CMP, Addressing::Absolute, CYCLE[index]),
            0xDD => (Instruction::CMP, Addressing::AbsoluteX, CYCLE[index]),
            0xD9 => (Instruction::CMP, Addressing::AbsoluteY, CYCLE[index]),
            0xC1 => (Instruction::CMP, Addressing::IndirectX, CYCLE[index]),
            0xD1 => (Instruction::CMP, Addressing::IndirectY, CYCLE[index]),
            //CPX
            0xE0 => (Instruction::CPX, Addressing::Immediate, CYCLE[index]),
            0xE4 => (Instruction::CPX, Addressing::ZeroPage, CYCLE[index]),
            0xEC => (Instruction::CPX, Addressing::Absolute, CYCLE[index]),
            //CPY
            0xC0 => (Instruction::CPY, Addressing::Immediate, CYCLE[index]),
            0xC4 => (Instruction::CPY, Addressing::ZeroPage, CYCLE[index]),
            0xCC => (Instruction::CPY, Addressing::Absolute, CYCLE[index]),
            //INC
            0xE6 => (Instruction::INC, Addressing::ZeroPage, CYCLE[index]),
            0xF6 => (Instruction::INC, Addressing::ZeroPageX, CYCLE[index]),
            0xEE => (Instruction::INC, Addressing::Absolute, CYCLE[index]),
            0xFE => (Instruction::INC, Addressing::AbsoluteX, CYCLE[index]),
            //DEC
            0xC6 => (Instruction::DEC, Addressing::ZeroPage, CYCLE[index]),
            0xD6 => (Instruction::DEC, Addressing::ZeroPageX, CYCLE[index]),
            0xCE => (Instruction::DEC, Addressing::Absolute, CYCLE[index]),
            0xDE => (Instruction::DEC, Addressing::AbsoluteX, CYCLE[index]),
            //INX
            0xE8 => (Instruction::INX, Addressing::Implied, CYCLE[index]),
            //DEX
            0xCA => (Instruction::DEX, Addressing::Implied, CYCLE[index]),
            //INY
            0xC8 => (Instruction::INY, Addressing::Implied, CYCLE[index]),
            //DEY
            0x88 => (Instruction::DEY, Addressing::Implied, CYCLE[index]),
            //CLC
            0x18 => (Instruction::CLC, Addressing::Implied, CYCLE[index]),
            //SEC
            0x38 => (Instruction::SEC, Addressing::Implied, CYCLE[index]),
            //CLI
            0x58 => (Instruction::CLI, Addressing::Implied, CYCLE[index]),
            //SEI
            0x78 => (Instruction::SEI, Addressing::Implied, CYCLE[index]),
            //CLD
            0xD8 => (Instruction::CLD, Addressing::Implied, CYCLE[index]),
            //SED
            0xF8 => (Instruction::SED, Addressing::Implied, CYCLE[index]),
            //CLV
            0xB8 => (Instruction::CLV, Addressing::Implied, CYCLE[index]),
            //LDA
            0xA9 => (Instruction::LDA, Addressing::Immediate, CYCLE[index]),
            0xA5 => (Instruction::LDA, Addressing::ZeroPage, CYCLE[index]),
            0xB5 => (Instruction::LDA, Addressing::ZeroPageX, CYCLE[index]),
            0xAD => (Instruction::LDA, Addressing::Absolute, CYCLE[index]),
            0xBD => (Instruction::LDA, Addressing::AbsoluteX, CYCLE[index]),
            0xB9 => (Instruction::LDA, Addressing::AbsoluteY, CYCLE[index]),
            0xA1 => (Instruction::LDA, Addressing::IndirectX, CYCLE[index]),
            0xB1 => (Instruction::LDA, Addressing::IndirectY, CYCLE[index]),
            //LDX
            0xA2 => (Instruction::LDX, Addressing::Immediate, CYCLE[index]),
            0xA6 => (Instruction::LDX, Addressing::ZeroPage, CYCLE[index]),
            0xB6 => (Instruction::LDX, Addressing::ZeroPageY, CYCLE[index]),
            0xAE => (Instruction::LDX, Addressing::Absolute, CYCLE[index]),
            0xBE => (Instruction::LDX, Addressing::AbsoluteY, CYCLE[index]),
            //LDY
            0xA0 => (Instruction::LDY, Addressing::Immediate, CYCLE[index]),
            0xA4 => (Instruction::LDY, Addressing::ZeroPage, CYCLE[index]),
            0xB4 => (Instruction::LDY, Addressing::ZeroPageX, CYCLE[index]),
            0xAC => (Instruction::LDY, Addressing::Absolute, CYCLE[index]),
            0xBC => (Instruction::LDY, Addressing::AbsoluteX, CYCLE[index]),
            //STA
            0x85 => (Instruction::STA, Addressing::Immediate, CYCLE[index]),
            0x95 => (Instruction::STA, Addressing::ZeroPage, CYCLE[index]),
            0x8D => (Instruction::STA, Addressing::ZeroPageX, CYCLE[index]),
            0x9D => (Instruction::STA, Addressing::ZeroPageY, CYCLE[index]),
            0x99 => (Instruction::STA, Addressing::Absolute, CYCLE[index]),
            0x81 => (Instruction::STA, Addressing::AbsoluteX, CYCLE[index]),
            0x91 => (Instruction::STA, Addressing::AbsoluteY, CYCLE[index]),
            //STX
            0x86 => (Instruction::STX, Addressing::ZeroPage, CYCLE[index]),
            0x96 => (Instruction::STX, Addressing::ZeroPageY, CYCLE[index]),
            0x8E => (Instruction::STX, Addressing::Absolute, CYCLE[index]),
            //STY
            0x84 => (Instruction::STY, Addressing::ZeroPage, CYCLE[index]),
            0x94 => (Instruction::STY, Addressing::ZeroPageX, CYCLE[index]),
            0x8C => (Instruction::STY, Addressing::Absolute, CYCLE[index]),
            //TAX
            0xAA=> (Instruction::TAX, Addressing::Implied, CYCLE[index]),
            //TXA
            0x8A=> (Instruction::TXA, Addressing::Implied, CYCLE[index]),
            //TAY
            0xA8 => (Instruction::TAY, Addressing::Implied, CYCLE[index]),
            //TYA
            0x98 => (Instruction::TYA, Addressing::Implied, CYCLE[index]),
            //TXS
            0x9A => (Instruction::TXS, Addressing::Implied, CYCLE[index]),
            //TSX
            0xBA => (Instruction::TSX, Addressing::Implied, CYCLE[index]),
            //PHA
            0x48 => (Instruction::PHA, Addressing::Implied, CYCLE[index]),
            //PLA
            0x68 => (Instruction::PLA, Addressing::Implied, CYCLE[index]),
            //PHP
            0x08 => (Instruction::PHP, Addressing::Implied, CYCLE[index]),
            //PLP
            0x28 => (Instruction::PLP, Addressing::Implied, CYCLE[index]),
            //NOP
            0xEA => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            /* Opecodes below isn't not official */
            // NOP
            0x1A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x3A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x5A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x7A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0xDA => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0xFA => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x02 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x12 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x22 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x32 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x42 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x52 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x62 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x72 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0x92 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0xB2 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0xD2 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            0xF2 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //NOPD
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPD, Addressing::Implied, CYCLE[index]),
            ////NOPI
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),
            //0x => (Instruction::NOPI, Addressing::Implied, CYCLE[index]),

            //0x => (Instruction::, Addressing::, CYCLE[index]),
            //0x => (Instruction::, Addressing::, CYCLE[index]),
            _ => (Instruction::Unknown, Addressing::Unknown, 0)
        }
    }
}
