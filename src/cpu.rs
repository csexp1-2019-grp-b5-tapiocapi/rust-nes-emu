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

    fn reset(&mut self) {
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
                println!("{} {} ", lower, upper);
                ReadResult::Addr(bit)
            }
            ReadSize::Byte => ReadResult::Data(bus.read_by_cpu(addr)),
        }
    }

    // fetch opcode (8-bit)
    fn fetch(&mut self) -> u8 {
        if let ReadResult::Data(i) = self.read(self.pc, ReadSize::Byte) {
            i
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    fn fetch_operand(&self) {}

    fn exec(&self) {}

    pub fn run(&mut self) {
        self.reset();
        println!("{:x}", self.pc);
    }
}
