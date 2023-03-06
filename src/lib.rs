pub struct Chip8 {
    /// 4kB address space
    mem: [u8; 0x1000],
    /// 64x32 display, each bit is a pixel
    disp: [u64; 32],
    /// Index register
    i: u16,
    /// Delay timer
    pub dt: u8,
    /// Sound timer
    pub st: u8,
    reg: [u8; 16],
    // Program counter
    pc: u16,
    stack: [u16; 16],
    /// Stack pointer
    sp: u8,
    pub keys: [bool; 16],
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut mem = [0; 0x1000];
        // chip 8 font
        mem[0x50..0xA0].copy_from_slice(&[
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]);
        let disp = [0; 32];
        let i = 0;
        let dt = 0;
        let st = 0;
        let reg = [0; 16];
        let pc = 0x200;
        let stack = [0; 16];
        let sp = 0;
        let keys = [false; 16];
        Self {
            mem,
            disp,
            i,
            dt,
            st,
            reg,
            stack,
            sp,
            keys,
            pc,
        }
    }
}

impl Chip8 {
    pub fn load_program<S: AsRef<[u8]>>(&mut self, program: S) -> std::io::Result<()> {
        use std::io::{Error, ErrorKind};
        let program = program.as_ref();
        if program.len() > 0x1000 - 0x200 {
            Err(Error::new(
                ErrorKind::Other,
                "CHIP-8 program must be 3,584 (0xE00) bytes or smaller.",
            ))
        } else {
            self.mem[0x200..0x200 + program.len()].copy_from_slice(program);
            Ok(())
        }
    }

    pub fn get_px(&self, x: u16, y: u16) -> bool {
        self.disp[(y % 32) as usize] >> x % 64 & 1 == 1
    }

    fn toggle_px(&mut self, x: u16, y: u16) {
        self.disp[(y % 32) as usize] ^= 1 << x % 64
    }

    fn fetch(&mut self) -> (u16, u16, u16, u16) {
        let a = self.mem[self.pc as usize] as u16;
        let b = self.mem[self.pc as usize + 1] as u16;
        self.pc += 2;
        (a >> 4, a & 0xf, b >> 4, b & 0xf)
    }

    pub fn cycle(&mut self) {
        match self.fetch() {
            (0x0, 0x0, 0xe, 0x0) => self.op_00e0(),
            (0x0, 0x0, 0xe, 0xe) => self.op_00ee(),
            (1, a, b, c) => self.op_1nnn(a << 8 | b << 4 | c),
            (2, a, b, c) => self.op_2nnn(a << 8 | b << 4 | c),
            (3, x, a, b) => self.op_3xnn(x, (a << 4 | b) as u8),
            (4, x, a, b) => self.op_4xnn(x, (a << 4 | b) as u8),
            (5, x, y, 0) => self.op_5xy0(x, y),
            (9, x, y, 0) => self.op_9xy0(x, y),
            (6, x, a, b) => self.op_6xnn(x, (a << 4 | b) as u8),
            (7, x, a, b) => self.op_7xnn(x, (a << 4 | b) as u8),
            (8, x, y, 0) => self.op_8xy0(x, y),
            (8, x, y, 1) => self.op_8xy1(x, y),
            (8, x, y, 2) => self.op_8xy2(x, y),
            (8, x, y, 3) => self.op_8xy3(x, y),
            (8, x, y, 4) => self.op_8xy4(x, y),
            (8, x, y, 5) => self.op_8xy5(x, y),
            (8, x, y, 6) => self.op_8xy6(x, y),
            (8, x, y, 7) => self.op_8xy7(x, y),
            (8, x, y, 0xe) => self.op_8xye(x, y),
            (0xa, a, b, c) => self.op_annn(a << 8 | b << 4 | c),
            (0xb, a, b, c) => self.op_bnnn(a << 8 | b << 4 | c),
            (0xc, x, a, b) => self.op_cxnn(x, (a << 4 | b) as u8),
            (0xd, x, y, n) => self.op_dxyn(x, y, n),
            (0xe, x, 0x9, 0xe) => self.op_ex9e(x),
            (0xe, x, 0xa, 0x1) => self.op_exa1(x),
            (0xf, x, 0x0, 0x7) => self.op_fx07(x),
            (0xf, x, 0x0, 0xa) => self.op_fx0a(x),
            (0xf, x, 0x1, 0x5) => self.op_fx15(x),
            (0xf, x, 0x1, 0xe) => self.op_fx1e(x),
            (0xf, x, 0x2, 0x9) => self.op_fx29(x),
            (0xf, x, 0x3, 0x3) => self.op_fx33(x),
            (0xf, x, 0x5, 0x5) => self.op_fx55(x),
            (0xf, x, 0x6, 0x5) => self.op_fx65(x),
            _ => (),
        }
    }

    fn op_dxyn(&mut self, x: u16, y: u16, height: u16) {
        println!("drw V{:X}, V{:X}, {}", x, y, height);

        self.reg[0xf] = 0;
        let x = (self.reg[x as usize] % 64) as u16;
        let y = (self.reg[y as usize] % 32) as u16;

        for cy in 0..height {
            let row = self.mem[self.i as usize + cy as usize];
            for cx in 0..8 {
                if x + cx > 63 {
                    break;
                }
                if row & 1 << (7 - cx) != 0 {
                    if self.get_px(x + cx, y + cy) {
                        self.reg[0xf] = 1;
                    }
                    self.toggle_px(x + cx, y + cy);
                }
            }
        }
    }

    fn op_00e0(&mut self) {
        println!("cls");
        self.disp.fill(0);
    }

    fn op_00ee(&mut self) {
        println!("ret");
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self, target: u16) {
        println!("jmp {}", target);
        self.pc = target;
    }

    fn op_2nnn(&mut self, target: u16) {
        println!("call #{}", target);
        self.stack[self.sp as usize] = self.pc;
        self.pc = target;
        self.sp += 1;
    }

    fn op_3xnn(&mut self, x: u16, imm: u8) {
        println!("sne V{:X}, {}", x, imm);
        if self.reg[x as usize] == imm {
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self, x: u16, imm: u8) {
        println!("sne V{:X}, {}", x, imm);
        if self.reg[x as usize] != imm {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x: u16, y: u16) {
        println!("se V{:X}, V{:X}", x, y);
        if self.reg[x as usize] == self.reg[y as usize] {
            self.pc += 2;
        }
    }

    fn op_9xy0(&mut self, x: u16, y: u16) {
        println!("sne V{:X}, V{:X}", x, y);
        if self.reg[x as usize] != self.reg[y as usize] {
            self.pc += 2;
        }
    }

    fn op_6xnn(&mut self, reg: u16, val: u8) {
        println!("ld V{:X}, {}", reg, val);
        self.reg[reg as usize] = val;
    }

    fn op_7xnn(&mut self, reg: u16, addend: u8) {
        println!("add V{:X}, {}", reg, addend);
        self.reg[reg as usize] = self.reg[reg as usize].wrapping_add(addend);
    }

    fn op_8xy0(&mut self, x: u16, y: u16) {
        println!("ld V{:X}, V{:X}", x, y);
        self.reg[x as usize] = self.reg[y as usize];
    }

    fn op_8xy1(&mut self, x: u16, y: u16) {
        println!("or V{:X}, V{:X}", x, y);
        self.reg[x as usize] |= self.reg[y as usize];
    }

    fn op_8xy2(&mut self, x: u16, y: u16) {
        println!("and V{:X}, V{:X}", x, y);
        self.reg[x as usize] &= self.reg[y as usize];
    }

    fn op_8xy3(&mut self, x: u16, y: u16) {
        println!("xor V{:X}, V{:X}", x, y);
        self.reg[x as usize] ^= self.reg[y as usize];
    }

    fn op_8xy4(&mut self, x: u16, y: u16) {
        let (x, y) = (x as usize, y as usize);
        println!("or V{:X}, V{:X}", x, y);
        match self.reg[x].checked_add(self.reg[y]) {
            Some(sum) => self.reg[x] = sum,
            None => {
                self.reg[0xf] = 1;
                self.reg[x] = self.reg[x].wrapping_add(self.reg[y]);
            }
        }
    }

    fn op_8xy5(&mut self, x: u16, y: u16) {
        let (x, y) = (x as usize, y as usize);
        println!("sub V{:X}, V{:X}", x, y);
        self.reg[0xf] = 0;
        if self.reg[x] > self.reg[y] {
            self.reg[0xf] = 1;
        }
        self.reg[x] = self.reg[x].wrapping_sub(self.reg[y]);
    }

    fn op_8xy6(&mut self, x: u16, y: u16) {
        let (x, y) = (x as usize, y as usize);
        println!("shl V{:X}, V{:X}", x, y);
        self.reg[x] = self.reg[y] >> 1;
        self.reg[0xf] = self.reg[y] & 1;
    }

    fn op_8xy7(&mut self, x: u16, y: u16) {
        let (x, y) = (x as usize, y as usize);
        println!("subn V{:X}, V{:X}", x, y);
        self.reg[0xf] = 0;
        if self.reg[y] > self.reg[x] {
            self.reg[0xf] = 1;
        }
        self.reg[x] = self.reg[y].wrapping_sub(self.reg[x]);
    }

    fn op_8xye(&mut self, x: u16, y: u16) {
        let (x, y) = (x as usize, y as usize);
        println!("shr V{:X}, V{:X}", x, y);
        self.reg[x] = self.reg[y] << 1;
        self.reg[0xf] = self.reg[y] >> 7;
    }

    fn op_annn(&mut self, i: u16) {
        println!("ld I, {:#x}", i);
        self.i = i;
    }

    fn op_bnnn(&mut self, target: u16) {
        println!("jmp V0 + {}", target);
        self.pc = self.reg[0] as u16 + target;
    }

    fn op_cxnn(&mut self, x: u16, mask: u8) {
        self.reg[x as usize] = rand::random::<u8>() & mask;
    }

    fn op_ex9e(&mut self, x: u16) {
        println!("skd K{:X}", x);
        if self.keys[self.reg[x as usize] as usize] {
            self.pc += 2;
        }
    }

    fn op_exa1(&mut self, x: u16) {
        println!("sku K{:X}", x);
        if !self.keys[self.reg[x as usize] as usize] {
            self.pc += 2;
        }
    }

    fn op_fx07(&mut self, x: u16) {
        println!("ld V{:X}, DT", x);
        self.reg[x as usize] = self.dt;
    }

    fn op_fx0a(&mut self, x: u16) {
        println!("wfk K{:X}", x);
        for i in 0..16 {
            if self.keys[i] {
                self.reg[x as usize] = i as u8;
                return
            }
        }
        self.pc -= 2;
    }

    fn op_fx15(&mut self, x: u16) {
        println!("ld DT, V{:X}", x);
        self.dt = self.reg[x as usize];
    }

    fn op_fx1e(&mut self, x: u16) {
        println!("add I, V{:X}", x);
        self.i += self.reg[x as usize] as u16;
    }

    fn op_fx29(&mut self, x: u16) {
        println!("ld F, V{:X}", x);
        self.i = 0x50 + x * 5;
    }

    fn op_fx33(&mut self, x: u16) {
        println!("ld B, V{:X}", x);
        let vx = self.reg[x as usize];
        let i = self.i as usize;
        self.mem[i] = vx / 100;
        self.mem[i + 1] = (vx % 100 - vx % 10) / 10;
        self.mem[i + 2] = vx % 10;
    }

    fn op_fx55(&mut self, x: u16) {
        println!("ld [I], V{:X}", x);
        for i in 0..=x as usize {
            self.mem[self.i as usize + i] = self.reg[i];
        }
    }

    fn op_fx65(&mut self, x: u16) {
        println!("ld V{:X}, [I]", x);
        for i in 0..=x as usize {
            self.reg[i] = self.mem[self.i as usize + i];
        }
    }
}
