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
}

enum Addressing {
    Accumlator,
    Immediate,
    Absolute,
    ZeroPage,
    IndexedZeroPage,
    IndexedAbsolute,
    Implied,
    Relative,
    IndexedIndirect,
    IndirectIndexed,
    AbsoluteIndirect
}

enum ReadSize {
    Word,
    Byte,
}

enum ReadResult {
    Data(u8),
    Addr(u16),
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
            self.pc += if self.pc < 0xFFFF { 1 } else {0};
            println!("{:x}", self.pc);
            i
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    fn fetch_operand(&self) {}

    fn exec(&self) {}

    pub fn run(&mut self) {
        let opcode = self.fetch();
        println!("{:x}", opcode);
    }
}
