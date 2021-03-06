use crate::cpu_bus;

pub struct Cpu {
    regs: Registers,
    bus: cpu_bus::CpuBus,
}

struct Registers {
    pub a: u8,   // accumlator register
    pub x: u8,   // index register
    pub y: u8,   // index register
    pub sp: u16, // stack pointer       (Begin from 0x1FD) Upper Bit is fixed to 0x01
    pub pc: u16, // program counter
    pub p: Status,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sp: 0x01FF,
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl From<Status> for u8 {
    fn from(v: Status) -> u8 {
        ((v.negative as u8) << 7)
            | ((v.overflow as u8) << 6)
            | ((v.reserved as u8) << 5)
            | ((v.break_mode as u8) << 4)
            | ((v.decimal as u8) << 3)
            | ((v.interrupt as u8) << 2)
            | ((v.zero as u8) << 1)
            | ((v.carry as u8) << 0)
    }
}

impl From<u8> for Status {
    fn from(v: u8) -> Status {
        Status {
            negative: (v & (1 << 7)) != 0,
            overflow: (v & (1 << 6)) != 0,
            reserved: (v & (1 << 5)) != 0,
            break_mode: (v & (1 << 4)) != 0,
            decimal: (v & (1 << 3)) != 0,
            interrupt: (v & (1 << 2)) != 0,
            zero: (v & (1 << 1)) != 0,
            carry: (v & (1 << 0)) != 0,
        }
    }
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

#[derive(Debug)]
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
    //NOPI,
    //NOPD,
}

#[derive(Debug)]
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
}

enum ReadSize {
    Word,
    Byte,
}

impl Cpu {
    pub fn new(cpu_bus: cpu_bus::CpuBus) -> Cpu {
        Cpu {
            regs: Default::default(),
            bus: cpu_bus,
        }
    }

    pub fn reset(&mut self) {
        self.regs = Default::default();
        self.regs.pc = self.read(0xFFFC, ReadSize::Word);
    }

    pub fn nmi_handler(&mut self) {
        self.regs.p.break_mode = false;
        self.push(((self.regs.pc & 0xFF00) >> 8) as u8);
        self.push((self.regs.pc & 0x00FF) as u8);
        self.push_status();
        self.regs.p.interrupt = true;
        self.regs.pc = self.read(0xFFFA, ReadSize::Word);
        println!("NMIdesu {:x}", self.regs.pc);
    }

    fn read(&mut self, addr: u16, size: ReadSize) -> u16 {
        let bus = &mut self.bus;
        match size {
            ReadSize::Word => {
                let lower = bus.read_by_cpu(addr);
                let upper = bus.read_by_cpu(addr + 0x0001);
                let mut byte = (upper as u16) << 8;
                byte |= lower as u16;
                byte
            }
            ReadSize::Byte => bus.read_by_cpu(addr) as u16,
        }
    }

    fn push(&mut self, data: u8) {
        self.bus.write_by_cpu(self.regs.sp, data);
        self.regs.sp -= 1;
    }

    fn push_status(&mut self) {
        self.push(self.regs.p.clone().into());
    }

    fn pop_status(&mut self) {
        self.regs.p = self.pop().into();
    }

    fn pop(&mut self) -> u8 {
        self.regs.sp += 1;
        let data = self.bus.read_by_cpu(self.regs.sp);
        data
    }

    // fetch opcode (8-bit)
    fn fetch(&mut self) -> u16 {
        let data = self.read(self.regs.pc, ReadSize::Byte);
        //println!("fetch 0x{:x} : {:x}", self.regs.pc, data);
        self.regs.pc += if self.regs.pc < 0xFFFF { 1 } else { 0 };
        data
    }

    fn fetch_addr(&mut self) -> u16 {
        let lower_byte = self.fetch();
        let upper_byte = self.fetch();
        //println!("fetch {:x} {:x}", upper_byte, lower_byte);
        ((upper_byte as u16) << 8) | lower_byte as u16
    }

    fn fetch_operand(&mut self, addressing: &Addressing) -> u16 {
        match addressing {
            Addressing::Accumlator => 0,
            Addressing::Immediate => self.fetch(),
            Addressing::Absolute => self.fetch_addr(),
            Addressing::ZeroPage => self.fetch() as u16,
            Addressing::ZeroPageX => (self.fetch() as u16 + self.regs.x as u16) & 0xFF,
            Addressing::ZeroPageY => (self.fetch() as u16 + self.regs.y as u16) & 0xFF,
            Addressing::AbsoluteX => self.fetch_addr() + self.regs.x as u16,
            Addressing::AbsoluteY => self.fetch_addr() + self.regs.y as u16,
            Addressing::Implied => 0,
            Addressing::Relative => {
                let base = self.fetch() as u16;
                if base < 0x80 {
                    base + self.regs.pc
                } else {
                    base + self.regs.pc - 256
                }
            }
            Addressing::Indirect => {
                let addr = self.fetch_addr();
                let lower_byte = self.read(addr, ReadSize::Byte);
                let upper_byte = self.read(addr + 1, ReadSize::Byte);
                (upper_byte << 8) | lower_byte
            }
            Addressing::IndirectX => {
                let addr = (self.fetch() + self.regs.x as u16) & 0xFF;
                let lower_byte = self.read(addr, ReadSize::Byte);
                let upper_byte = self.read(addr + 1, ReadSize::Byte);
                (upper_byte << 8) | lower_byte
            }
            Addressing::IndirectY => {
                let addr = (self.fetch() as u16) & 0xFF;
                let lower_byte = self.read(addr, ReadSize::Byte);
                let upper_byte = self.read(addr + 1, ReadSize::Byte);
                (upper_byte << 8) | lower_byte + self.regs.y as u16
            }
        }
    }

    fn check_negative(&self, register: &u8) -> bool {
        (register & (1 << 7)) >> 7 == 1
    }

    fn checked_add(reg: u8, operand: u8, carry: bool) -> (u8, bool) {
        let carry = if carry { 1 } else { 0 };
        let result = (reg as i8).checked_add(operand as i8 + carry);

        match result {
            Some(res) => (res as u8, false),
            None => ((reg as i16 + operand as i16 + carry as i16) as u8, true),
        }
    }

    fn checked_sub(reg: u8, operand: u8, carry: bool) -> (u8, bool) {
        let carry = if carry { 1 } else { 0 };
        let result = (reg as i8).checked_sub(operand as i8 + carry);

        match result {
            Some(res) => (res as u8, false),
            None => ((reg as i16 - operand as i16 - carry as i16) as u8, true),
        }
    }

    fn exec(&mut self, instruction: &Instruction, addressing: &Addressing, operand: u16) {
        match instruction {
            Instruction::ADC => {
                let data = match addressing {
                    Addressing::Immediate => operand as u8,
                    _ => self.read(operand, ReadSize::Byte) as u8,
                };

                let (reg, overflow) = Self::checked_add(self.regs.a, data, self.regs.p.carry);
                self.regs.a = reg;
                self.regs.p.overflow = overflow;

                self.regs.p.negative = self.check_negative(&self.regs.a);
                self.regs.p.zero = self.regs.a == 0;
            }
            Instruction::SBC => {
                let data = match addressing {
                    Addressing::Immediate => operand as u8,
                    _ => self.read(operand, ReadSize::Byte) as u8,
                };

                let (reg, overflow) = Self::checked_sub(self.regs.a, data, self.regs.p.carry);
                self.regs.a = reg;
                self.regs.p.overflow = overflow;

                self.regs.p.negative = self.check_negative(&self.regs.a);
                self.regs.p.zero = self.regs.a == 0;
            }
            Instruction::AND => {
                //print!("AND ");
                match addressing {
                    Addressing::Immediate => {
                        self.regs.a &= operand as u8;
                        //print!("#{:x}", operand as u8);
                    }
                    _ => {
                        let data = self.read(operand, ReadSize::Byte) as u8;
                        self.regs.a &= data;
                        //print!("#{:x}", data);
                    }
                }
                self.regs.p.negative = self.check_negative(&self.regs.a);
                self.regs.p.zero = self.regs.a == 0;
            }
            Instruction::ORA => {
                //print!("ORA ");
                match addressing {
                    Addressing::Immediate => {
                        self.regs.a |= operand as u8;
                        //print!("#{:x}", operand);
                    }
                    _ => {
                        let data = self.read(operand, ReadSize::Byte) as u8;
                        self.regs.a |= data;
                        //print!("#{:x}", data);
                    }
                }
                self.regs.p.negative = self.check_negative(&self.regs.a);
                self.regs.p.zero = self.regs.a == 0;
            }
            Instruction::EOR => {
                //print!("EOR ");
                match addressing {
                    Addressing::Immediate => {
                        self.regs.a ^= operand as u8;
                        //print!("#{:x}", operand);
                    }
                    _ => {
                        let data = self.read(operand, ReadSize::Byte) as u8;
                        self.regs.a ^= data;
                        //print!("#{:x}", data);
                    }
                }
                self.regs.p.negative = self.check_negative(&self.regs.a);
                self.regs.p.zero = self.regs.a == 0;
            }
            Instruction::ASL => {
                //print!("ASL ");
                match addressing {
                    Addressing::Accumlator => {
                        self.regs.p.carry = ((self.regs.a & (1 << 7)) >> 7) != 0;
                        self.regs.a = ((self.regs.a as i8) << 1) as u8;
                    }
                    _ => {
                        self.regs.p.carry = (((operand as u8) & (1 << 7)) >> 7) != 0;
                        self.regs.a = ((operand as i8) << 1) as u8;
                    }
                }
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
            }
            Instruction::LSR => {
                //print!("LSR ");
                match addressing {
                    Addressing::Accumlator => {
                        self.regs.p.carry = (self.regs.a & 1) != 0;
                        self.regs.a = self.regs.a >> 1;
                    }
                    _ => {
                        self.regs.p.carry = (operand & 1) != 0;
                        self.regs.a = (operand as u8) >> 1;
                    }
                }
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
            }
            Instruction::ROL => {
                //print!("ROL");
                match addressing {
                    Addressing::Accumlator => {
                        self.regs.p.carry = ((self.regs.a & (1 << 7)) >> 7) != 0;
                        self.regs.a = self.regs.a.rotate_left(1);
                    }
                    _ => {
                        self.regs.p.carry = (((operand as u8) & (1 << 7)) >> 7) != 0;
                        self.regs.a = (operand as u8).rotate_left(1);
                    }
                }
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
            }
            Instruction::ROR => {
                //print!("ROR ");
                match addressing {
                    Addressing::Accumlator => {
                        self.regs.p.carry = (self.regs.a & 1) != 0;
                        self.regs.a = self.regs.a.rotate_right(1);
                    }
                    _ => {
                        self.regs.p.carry = (operand & 1) != 0;
                        self.regs.a = (operand as u8).rotate_right(1);
                    }
                }
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
            }
            Instruction::BCC => {
                //print!("BCC ");
                self.regs.pc = if !self.regs.p.carry {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BCS => {
                //print!("BCS ");
                self.regs.pc = if self.regs.p.carry {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BEQ => {
                //print!("BEQ ");
                self.regs.pc = if self.regs.p.zero {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BNE => {
                //println!("BNE ${:x} ", operand);
                self.regs.pc = if !self.regs.p.zero {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BVC => {
                //print!("BVC ");
                self.regs.pc = if !self.regs.p.overflow {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BVS => {
                //print!("BVS ");
                self.regs.pc = if self.regs.p.overflow {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BPL => {
                //print!("BPL ");
                self.regs.pc = if !self.regs.p.negative {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BMI => {
                //print!("BMI ");
                self.regs.pc = if self.regs.p.negative {
                    //print!("{:x} -> pc:{:x}", operand, self.regs.pc);
                    operand
                } else {
                    self.regs.pc
                };
            }
            Instruction::BIT => {
                let target = self.read(operand, ReadSize::Byte) as u8;
                let result = self.regs.a & target;
                self.regs.p.zero = result == 0;
                self.regs.p.negative = target & (1 << 7) == 0b1000_0000;
                self.regs.p.overflow = target & (1 << 6) == 0b0100_0000;
                //print!("BIT ");
            }
            Instruction::JMP => {
                self.regs.pc = operand;
                //print!("JMP {:x} -> pc:{:x}", operand, self.regs.pc);
            }
            Instruction::JSR => {
                self.push(((self.regs.pc & 0xFF00) >> 8) as u8);
                self.push((self.regs.pc & 0xFF) as u8);
                //self.print_stack();
                self.regs.pc = operand;
                //print!("JSR");
            }
            Instruction::RTS => {
                //self.print_stack();
                let lower = self.pop() as u16;
                let upper = self.pop() as u16;
                self.regs.pc = (upper << 8) | lower;
                //print!("RTS -> {:x}", self.regs.pc);
            }
            Instruction::BRK => {
                //print!("BRK");
                let interrupt = self.regs.p.interrupt;
                //self.regs.pc += 1;
                //let lower = (self.regs.pc >> 8) as u8;
                //self.push(lower);
                if !interrupt {
                    self.regs.p.break_mode = true;
                    self.regs.pc += 1;
                    self.push(((self.regs.pc & 0xFF00) >> 4) as u8);
                    self.push((self.regs.pc & 0xFF) as u8);
                    self.push_status();
                    self.regs.p.interrupt = true;
                    self.regs.pc = self.read(0xFFFE, ReadSize::Word);
                } else {
                    return;
                }
            }
            Instruction::RTI => {
                self.pop_status();
                let lower = self.pop() as u16;
                let upper = self.pop() as u16;
                self.regs.pc = ((upper) << 8) | lower;
                //print!("RTI");
            }
            Instruction::CMP => {
                let m = match addressing {
                    Addressing::Immediate => operand as u8,
                    _ => self.read(operand, ReadSize::Byte) as u8,
                };
                self.regs.p.carry = self.regs.a >= m;
                self.regs.p.zero = self.regs.a == m;
                self.regs.p.negative = (self.regs.a as i8) > (m as i8);
                //print!("CMP");
            }
            Instruction::CPX => {
                let m = match addressing {
                    Addressing::Immediate => operand as u8,
                    _ => self.read(operand, ReadSize::Byte) as u8,
                };
                self.regs.p.carry = self.regs.x >= m;
                self.regs.p.zero = self.regs.x == m;
                self.regs.p.negative = self.regs.x > m;
                //print!("CPX");
            }
            Instruction::CPY => {
                let m = match addressing {
                    Addressing::Immediate => operand as u8,
                    _ => self.read(operand, ReadSize::Byte) as u8,
                };
                self.regs.p.carry = self.regs.y >= m;
                self.regs.p.zero = self.regs.y == m;
                self.regs.p.negative = self.regs.y > m;
                //print!("CPY");
            }
            Instruction::INC => {
                let data = self.read(operand, ReadSize::Byte);
                let result = (data as i16 + 1) as u8;
                self.bus.write_by_cpu(operand, result);
                self.regs.p.zero = result == 0;
                self.regs.p.negative = self.check_negative(&self.regs.x);
                //print!("INC");
            }
            Instruction::DEC => {
                //print!("DEC");
                let data = self.read(operand, ReadSize::Byte) as u8;
                let result = (data as i16 - 1) as u8;
                self.bus.write_by_cpu(operand, result);
                self.regs.p.zero = result == 0;
                self.regs.p.negative = self.check_negative(&result);
            }
            Instruction::INX => {
                //print!("INX null\n : x:{:x}+1 ->", self.regs.x);
                self.regs.x = (self.regs.x as i16 + 1) as u8;
                self.regs.p.zero = self.regs.x == 0;
                self.regs.p.negative = self.check_negative(&self.regs.x);
                //print!(" x:{:x}", self.regs.x);
            }
            Instruction::DEX => {
                //print!("DEX null\n : x:{:x}-1 ->", self.regs.x);
                self.regs.x = (self.regs.x as i16 - 1) as u8;
                self.regs.p.zero = self.regs.x == 0;
                self.regs.p.negative = self.check_negative(&self.regs.x);
                //print!(" x:{:x}", self.regs.x);
            }
            Instruction::INY => {
                //print!("INY null\n : y:{:x}+1 ->", self.regs.y);
                self.regs.y = (self.regs.y as i16 + 1) as u8;
                self.regs.p.zero = self.regs.y == 0;
                self.regs.p.negative = self.check_negative(&self.regs.y);
                //print!(" y:{:x}", self.regs.y);
            }
            Instruction::DEY => {
                //print!("DEY null\n : y:{:x}-1 ->", self.regs.y);
                self.regs.y = (self.regs.y as i16 - 1) as u8;
                self.regs.p.zero = self.regs.y == 0;
                self.regs.p.negative = self.check_negative(&self.regs.y);
            }
            Instruction::CLC => {
                self.regs.p.carry = false;
                //print!("CLC");
            }
            Instruction::SEC => {
                self.regs.p.carry = true;
                //print!("SEC");
            }
            Instruction::CLI => {
                self.regs.p.interrupt = false;
                //print!("CLI false -> p.interrupt");
            }
            Instruction::SEI => {
                self.regs.p.interrupt = true;
                //println!("SEI null");
                //print!(" :true -> p.interrupt");
            }
            Instruction::CLD => {
                self.regs.p.decimal = false;
                //print!("CLD false -> p.decimal");
            }
            Instruction::SED => {
                self.regs.p.decimal = true;
                //print!("SED true -> p.decimal");
            }
            Instruction::CLV => {
                self.regs.p.overflow = false;
                //print!("CLV false -> p.overflow");
            }
            Instruction::LDA => {
                //print!("LDA ");
                self.regs.a = match addressing {
                    Addressing::Immediate => {
                        //println!("#{:x}", operand);
                        operand as u8
                    }
                    _ => {
                        //println!("${:x}", operand);
                        self.read(operand, ReadSize::Byte) as u8
                    }
                };
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
                //print!(" :{:x} -> A:{:x}", operand, self.regs.a);
            }
            Instruction::LDX => {
                //print!("LDX ");
                self.regs.x = match addressing {
                    Addressing::Immediate => {
                        //println!("#{}", operand);
                        operand as u8
                    }
                    _ => {
                        //println!("${}", operand);
                        self.read(operand, ReadSize::Byte) as u8
                    }
                };
                self.regs.p.zero = self.regs.x == 0;
                self.regs.p.negative = self.check_negative(&self.regs.x);
                //print!(" : {:x} -> X", self.regs.x);
            }
            Instruction::LDY => {
                //print!("LDY ");
                self.regs.y = match addressing {
                    Addressing::Immediate => {
                        //println!("#{}", operand);
                        operand as u8
                    }
                    _ => {
                        //println!("${}", operand);
                        self.read(operand, ReadSize::Byte) as u8
                    }
                };
                self.regs.p.zero = self.regs.y == 0;
                self.regs.p.negative = self.check_negative(&self.regs.y);
                //print!(" : {} -> Y", self.regs.y);
            }
            Instruction::STA => {
                self.bus.write_by_cpu(operand, self.regs.a as u8);
                //print!("STA ${:x}\n a:{:x} -> {:x}", operand, self.regs.a, operand);
            }
            Instruction::STX => {
                self.bus.write_by_cpu(operand, self.regs.x as u8);
                //print!("STX x:{:x} -> {:x}", self.regs.x, operand);
            }
            Instruction::STY => {
                self.bus.write_by_cpu(operand, self.regs.y as u8);
                //print!("STY y:{:x} -> {:x}", self.regs.y, operand);
            }
            Instruction::TAX => {
                self.regs.x = self.regs.a;
                self.regs.p.zero = self.regs.x == 0;
                self.regs.p.negative = self.check_negative(&self.regs.x);
                //print!("TAX null: A:{:x} -> X", self.regs.a);
            }
            Instruction::TXA => {
                self.regs.a = self.regs.x;
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
                //print!("TXA");
            }
            Instruction::TAY => {
                self.regs.y = self.regs.a;
                self.regs.p.zero = self.regs.y == 0;
                self.regs.p.negative = self.check_negative(&self.regs.y);
                //print!("TAY");
            }
            Instruction::TYA => {
                self.regs.a = self.regs.y;
                self.regs.p.zero = self.regs.a == 0;
                self.regs.p.negative = self.check_negative(&self.regs.a);
                //print!("TYA");
            }
            Instruction::TSX => {
                self.regs.x = (self.regs.sp & 0xFF) as u8;
                self.regs.p.negative = self.check_negative(&self.regs.x);
                self.regs.p.zero = self.regs.x == 0;
                //print!("TSX: S(SP){:x} -> X:{:x}", self.regs.sp, self.regs.x);
            }
            Instruction::TXS => {
                self.regs.sp = (self.regs.x as u16) | 0x0100;
                //print!(
                //    "TXS null\n : X:{:x} -> S(SP):{:x}",
                //    self.regs.x, self.regs.sp
                //);
            }
            Instruction::PHA => {
                self.push(self.regs.a);
                //self.print_stack();
                //print!("PHA a:{:x} -> stack:{:x}", self.regs.a, self.regs.sp);
            }
            Instruction::PLA => {
                self.regs.a = self.pop();
                self.regs.p.negative = self.check_negative(&self.regs.a);
                self.regs.p.zero = self.regs.a == 0;
                //self.print_stack();
                //print!("PLA stack:{:x} -> A:{:x}", self.regs.sp, self.regs.a);
            }
            Instruction::PHP => {
                self.push_status();
                //print!("PHP");
            }
            Instruction::PLP => {
                self.pop_status();
                //print!("PLP");
            }
            Instruction::NOP => {
                //print!("NOP");
            }
        }
    }

    pub fn run(&mut self) {
        let nmi_int = *cpu_bus::NMI_INT.borrow();
        if nmi_int {
            println!("==NMI_INT==");
            self.nmi_handler();
            *cpu_bus::NMI_INT.borrow_mut() = false;
        }
        let pc_for_log = self.regs.pc;
        let opcode = self.fetch();
        let op_info = self.get_instruction_info(opcode);
        let operand = self.fetch_operand(&op_info.1);
        self.exec(&op_info.0, &op_info.1, operand);
        self.print_log(pc_for_log, opcode, operand, &op_info.0, &op_info.1);
    }

    fn print_log(
        &mut self,
        pc: u16,
        opcode: u16,
        operand: u16,
        inst: &Instruction,
        addr: &Addressing,
    ) {
        print!("0x{:x}: {:>02x} ", pc, opcode);
        match addr {
            Addressing::Accumlator => {
                print!("      | {:?}", inst);
            }
            Addressing::Immediate => {
                print!(
                    "{:>02x}    | {:?} #${:?}    ",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::Absolute => {
                print!(
                    "{:>02x} {:>02x} | {:?} ${:x}",
                    &self.read(pc + 1, ReadSize::Byte),
                    &self.read(pc + 2, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::AbsoluteX => {
                print!(
                    "{:>02x} {:>02x} | {:?} ${:x}, X",
                    &self.read(pc + 1, ReadSize::Byte),
                    &self.read(pc + 2, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::AbsoluteY => {
                print!(
                    "{:>02x} {:>02x} | {:?} ${:x}, Y  ",
                    &self.read(pc + 1, ReadSize::Byte),
                    &self.read(pc + 2, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::ZeroPage => {
                print!(
                    "{:>02x}    | {:?} ${:x}",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::ZeroPageX => {
                print!(
                    "{:>02x}    | {:?} ${:x}, X",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::ZeroPageY => {
                print!(
                    "{:>02x}    | {:?} ${:x}, X",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::Implied => {
                print!("      | {:?}", inst);
            }
            Addressing::Relative => {
                print!(
                    "{:>02x}    | {:?} ${:2x}    ",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    &self.read(pc + 1, ReadSize::Byte)
                );
            }
            Addressing::Indirect => {
                print!(
                    "{:>02x} {:>02x} | {:?} (${:x})",
                    &self.read(pc + 1, ReadSize::Byte),
                    &self.read(pc + 2, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::IndirectX => {
                print!(
                    "{:>02x}    | {:?} (${:2x}, X)",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    operand
                );
            }
            Addressing::IndirectY => {
                print!(
                    "{:>02x}    | {:?} (${:2x}), Y ",
                    &self.read(pc + 1, ReadSize::Byte),
                    inst,
                    operand
                );
            }
        }
        println!(
            "A: {:2x} X: {:2x} Y: {:2x} SP:{:4x}",
            &self.regs.a, &self.regs.x, &self.regs.y, &self.regs.sp,
        );
    }

    fn get_instruction_info(&self, opcode: u16) -> (Instruction, Addressing, u8) {
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
            0x85 => (Instruction::STA, Addressing::ZeroPage, CYCLE[index]),
            0x95 => (Instruction::STA, Addressing::ZeroPageX, CYCLE[index]),
            0x8D => (Instruction::STA, Addressing::Absolute, CYCLE[index]),
            0x9D => (Instruction::STA, Addressing::AbsoluteX, CYCLE[index]),
            0x99 => (Instruction::STA, Addressing::AbsoluteY, CYCLE[index]),
            0x81 => (Instruction::STA, Addressing::IndirectX, CYCLE[index]),
            0x91 => (Instruction::STA, Addressing::IndirectY, CYCLE[index]),
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
            /* Opecodes below are unofficial */
            // NOP
            //0x1A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x3A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x5A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x7A => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0xDA => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0xFA => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x02 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x12 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x22 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x32 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x42 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x52 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x62 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x72 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0x92 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0xB2 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0xD2 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
            //0xF2 => (Instruction::NOP, Addressing::Implied, CYCLE[index]),
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
            _ => panic!("{} unknown", opcode)
        }
    }

    fn print_stack(&mut self) {
        println!("<<<<<<<<<<<<<<<<<<<");
        let sp = 1 + self.regs.sp;
        for i in sp..0x0200 {
            let data = self.read(i, ReadSize::Byte);
            println!("sp:{:x} val:{:x}", i, data);
        }
        println!("<<<<<<<<<<<<<<<<<<<");
    }
}

#[cfg(test)]
mod tests {
    // FIXME: more tests

    use super::*;
    use crate::*;
    fn configure_cpu(prog: &[u8]) -> Cpu {
        let prom_buf = {
            let mut buf = [0; 0x8000];
            // copy the given program to the PROM
            buf[..prog.len()].copy_from_slice(&prog);

            // set the reset vector to 0x8000 (the beginning of the PROM)
            buf[0xFFFC - 0x8000] = 0x00;
            buf[0xFFFD - 0x8000] = 0x80;

            buf
        };

        let prom = rom::ProgramRom::new(&prom_buf);
        let crom = rom::CharacterRom::new(&[]);
        let wram = ram::Ram::new(0x0800);
        let ppu = ppu::Ppu::new(crom);

        let cpu_bus = cpu_bus::CpuBus::new(wram, prom, ppu);

        Cpu::new(cpu_bus)
    }

    #[test]
    fn test_status_type_conv() {
        let s1 = Status {
            negative: false,
            overflow: true,
            reserved: false,
            break_mode: true,
            decimal: false,
            interrupt: true,
            zero: false,
            carry: true,
        };
        assert_eq!(u8::from(s1), 0b0101_0101);

        let s2 = Status {
            negative: true,
            overflow: false,
            reserved: true,
            break_mode: false,
            decimal: true,
            interrupt: false,
            zero: true,
            carry: false,
        };
        assert_eq!(s2, Status::from(0b1010_1010));
    }

    #[test]
    fn test_status_push_pop() {
        let prog = [0x08, 0x28]; // PHP, PLP

        let mut cpu = configure_cpu(&prog);
        cpu.reset();

        let status = 0b0101_0101;
        cpu.regs.p = status.into();
        cpu.run();
        assert_eq!(cpu.regs.sp, 0x01FE); // sp should be incremented
        assert_eq!(cpu.regs.p, Status::from(status)); // status should not change after PHP
        assert_eq!(cpu.bus.read_by_cpu(cpu.regs.sp+1), status); // status should be pushed onto the stack

        cpu.regs.p = 0b1111_1111.into(); // alter status
        cpu.run();
        assert_eq!(cpu.regs.sp, 0x01FF); // sp should be decremented
        assert_eq!(cpu.regs.p, Status::from(status)); // status should be the original state after PLP
    }

    #[test]
    fn test_adc() {
        let prog = [0x69, 0x01, // ADC #$1 : Immediate
                    0x69, 0x01,
                    0x65, 0x00, // ADC $0 : Zero Page
                    0x65, 0x00,
                    0x75, 0x00, // ADC $0, X : Zero Page, X
                    0x75, 0x00,
                    0x6D, 0x00, 0x05, // ADC $0x500 : Absolute
                    0x6D, 0x00, 0x05,
                    0x7D, 0x00, 0x05, // ADC $0x500 : Absolute, X
                    0x7D, 0x00, 0x05, 
                    0x79, 0x00, 0x05, // ADC $0x500 : Absolute, Y
                    0x79, 0x00, 0x05, 
                    0x61, 0x00, // ADC ($0, X): Indirect, X
                    0x61, 0x00,  
                    0x71, 0x00, // ADC ($0), Y: Indirect, Y
                    0x71, 0x00,  
        ];

        let mut cpu = configure_cpu(&prog);
        cpu.reset();
        cpu.regs.a = 0x1;
        cpu.regs.x = 0x1;
        cpu.regs.y = 0x1;

        /**** Immediate ****/
        cpu.run(); // ADC #$1
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;

        /**** Zero Page ****/
        cpu.bus.write_by_cpu(0x0, 0x1);
        cpu.run(); // ADC $0
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;

        /**** Zero Page, X ****/
        cpu.bus.write_by_cpu(0x1, 0x1);
        cpu.run(); // ADC $0, X
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;

        /**** Absolute ****/
        cpu.bus.write_by_cpu(0x500, 0x1);
        cpu.run(); // ADC $500
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;
        cpu.bus.write_by_cpu(0x500, 0x0);

        /**** Absolute, X ****/
        cpu.bus.write_by_cpu(0x501, 0x1);
        cpu.run(); // ADC $500, X
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;
        cpu.bus.write_by_cpu(0x501, 0x0);

        /**** Absolute, Y ****/
        cpu.bus.write_by_cpu(0x501, 0x1);
        cpu.run(); // ADC $500, Y
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;
        cpu.bus.write_by_cpu(0x501, 0x0);

        /**** Indirect, X ****/
        cpu.bus.write_by_cpu(0x1, 0xA);
        cpu.bus.write_by_cpu(0x2, 0x0);
        cpu.bus.write_by_cpu(0xA, 0x1);
        cpu.run(); // ADC ($0, X);
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);

        cpu.regs.p.carry = false;
        cpu.regs.a = 0x1;
        cpu.bus.write_by_cpu(0x1, 0x0);
        cpu.bus.write_by_cpu(0x2, 0x0);
        cpu.bus.write_by_cpu(0xA, 0x0);

        /**** Indirect, Y ****/
        cpu.bus.write_by_cpu(0x0, 0x0);
        cpu.bus.write_by_cpu(0x1, 0x5);
        cpu.bus.write_by_cpu(0x501, 0x1);
        cpu.run(); // ADC ($0), Y;
        assert_eq!(cpu.regs.a, 0x2);

        cpu.regs.p.carry = true; // add carry;
        cpu.run();
        assert_eq!(cpu.regs.a, 0x4);
    }

    #[test]
    fn test_jmp() {
        let prog = [0x4C, 0x03, 0x80, //JMP $0x8003 : Absolute
                    0xEA,             //NOP (@0x8003)
                    0x6C, 0x0A, 0x80, //JMP ($0x800A) : Indirect
                    0x00, 0x00, 0x00, //dummy
                    0x0C,             //lower byte (@0x800A)
                    0x80,             //upper byte (@0x800B)
                    0xEA,             //NOP (@0x800C)
        ];

        let mut cpu = configure_cpu(&prog);
        cpu.reset();

        /**** Absolute ****/
        cpu.run(); //JMP $0x8003
        assert_eq!(cpu.regs.pc, 0x8003);
        assert_eq!(cpu.bus.read_by_cpu(cpu.regs.pc), 0xEA);

        /**** Indirect ****/
        cpu.run(); //NOP
        cpu.run(); //JMP (0x800A)
        assert_eq!(cpu.regs.pc, 0x800C);
        assert_eq!(cpu.bus.read_by_cpu(cpu.regs.pc), 0xEA);
    }

    #[test]
    fn test_subroutine_inst() {
        let prog = [0x20, 0x0A, 0x80, //JSR $0x800A : Absolute
                    0xEA,             //NOP (@0x8003)
                    0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00,
                    0x60,             //RTS : Implied
        ];
        let mut cpu = configure_cpu(&prog);
        cpu.reset();
        cpu.run(); //JSR $0x800A
        assert_eq!(cpu.regs.pc, 0x800A);
        assert_eq!(cpu.bus.read_by_cpu(cpu.regs.pc), 0x60);

        cpu.run(); //RTS
        assert_eq!(cpu.regs.pc, 0x8003);
        assert_eq!(cpu.bus.read_by_cpu(cpu.regs.pc), 0xEA);
    }

    #[test]
    fn test_load_inst() { //LDA LDX LDY
        //---- LDA ----//
        let prog = [0xA9, 0x1, //LDA #$1 : Immediate
                    0xA5, 0x0, //LDA $0 : Zero Page
                    0xB5, 0x0, //LDA $0, X : Zero Page, X
                    0xAD, 0x0, 0x0, //LDA $0x0000 : Absolute
                    0xBD, 0x0, 0x0, //LDA $0x0001, X : Absolute, X
                    0xB9, 0x0, 0x0, //LDA $0x0001, Y : Absolute, Y
                    0xA1, 0x0, //LDA ($0x0, X) : Indirect, X
                    0xB1, 0x0  //LDA ($0x0), Y
        ];
        let mut cpu = configure_cpu(&prog);
        cpu.reset();

        cpu.bus.write_by_cpu(0x0, 0x2);
        cpu.bus.write_by_cpu(0x1, 0x3);
        cpu.bus.write_by_cpu(0x2, 0x4);
        cpu.bus.write_by_cpu(0x403, 0xA);
        cpu.bus.write_by_cpu(0x304, 0xB);
        cpu.regs.x = 0x1;
        cpu.regs.y = 0x2;

        /**** Immediate ****/
        cpu.run(); //LDA #$1
        assert_eq!(cpu.regs.a, 0x1);

        /**** Zero Page ****/
        cpu.run(); //LDA $0
        assert_eq!(cpu.regs.a, 0x2);

        /**** Zero Page, X ****/
        cpu.run(); //LDA $0, X
        assert_eq!(cpu.regs.a, 0x3);

        /**** Absolute ****/
        cpu.run(); //LDA $0000
        assert_eq!(cpu.regs.a, 0x2);

        /**** Absolute, X ****/
        cpu.run(); //LDA $0000, X
        assert_eq!(cpu.regs.a, 0x3);

        /**** Absolute, Y ****/
        cpu.run(); //LDA $0000, Y
        assert_eq!(cpu.regs.a, 0x4);

        /**** Indirect, X ****/
        cpu.run(); //LDA ($0000, X)
        assert_eq!(cpu.regs.a, 0xA);

        /**** Indirect, Y ****/
        cpu.run(); //LDA ($0000, Y)
        assert_eq!(cpu.regs.a, 0xB);


        //---- LDX ----//
        let prog = [0xA2, 0x1, //LDX #$1 : Immediate
                    0xA6, 0x0, //LDX $0 : Zero Page
                    0xB6, 0x0, //LDX $0, Y : Zero Page, Y
                    0xAE, 0x0, 0x0, //LDX $0x0000 : Absolute
                    0xBE, 0x0, 0x0, //LDX $0x0001, Y : Absolute, Y
        ];

        let mut cpu = configure_cpu(&prog);
        cpu.reset();

        cpu.bus.write_by_cpu(0x0, 0x2);
        cpu.bus.write_by_cpu(0x1, 0x3);
        cpu.bus.write_by_cpu(0x2, 0x4);
        cpu.bus.write_by_cpu(0x403, 0xA);
        cpu.bus.write_by_cpu(0x304, 0xB);
        cpu.regs.y = 0x2;

        /**** Immediate ****/
        cpu.run(); //LDX #$1
        assert_eq!(cpu.regs.x, 0x1);

        /**** Zero Page ****/
        cpu.run(); //LDX $0
        assert_eq!(cpu.regs.x, 0x2);

        /**** Zero Page, Y ****/
        cpu.run(); //LDX $0
        assert_eq!(cpu.regs.x, 0x4);

        /**** Absolute ****/
        cpu.run(); //LDX $0000
        assert_eq!(cpu.regs.x, 0x2);

        /**** Absolute, Y ****/
        cpu.run(); //LDX $0000, Y
        assert_eq!(cpu.regs.x, 0x4);

        //---- LDY ----//
        let prog = [0xA0, 0x1, //LDX #$1 : Immediate
                    0xA4, 0x0, //LDX $0 : Zero Page
                    0xB4, 0x0, //LDX $0, X : Zero Page, X
                    0xAC, 0x0, 0x0, //LDX $0x0000 : Absolute
                    0xBC, 0x0, 0x0, //LDX $0x0001, X : Absolute, X
        ];

        let mut cpu = configure_cpu(&prog);
        cpu.reset();

        cpu.bus.write_by_cpu(0x0, 0x2);
        cpu.bus.write_by_cpu(0x1, 0x3);
        cpu.bus.write_by_cpu(0x2, 0x4);
        cpu.bus.write_by_cpu(0x403, 0xA);
        cpu.bus.write_by_cpu(0x304, 0xB);
        cpu.regs.x = 0x2;

        /**** Immediate ****/
        cpu.run(); //LDY #$1
        assert_eq!(cpu.regs.y, 0x1);

        /**** Zero Page ****/
        cpu.run(); //LDY $0
        assert_eq!(cpu.regs.y, 0x2);

        /**** Zero Page, X ****/
        cpu.run(); //LDY $0
        assert_eq!(cpu.regs.y, 0x4);

        /**** Absolute ****/
        cpu.run(); //LDY $0000
        assert_eq!(cpu.regs.y, 0x2);

        /**** Absolute, X ****/
        cpu.run(); //LDY $0000, X
        assert_eq!(cpu.regs.y, 0x4);

    }
}
