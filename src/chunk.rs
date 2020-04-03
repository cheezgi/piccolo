use crate::value::Value;

// TODO: change lines to a reasonable number type
/// Stores a piece of compiled Piccolo bytecode.
#[derive(Default)]
pub struct Chunk {
    pub(crate) data: Vec<u8>,
    pub(crate) lines: Vec<usize>,
    pub(crate) constants: Vec<Value>,
}

impl Chunk {
    pub(crate) fn write<T: Into<u8>>(&mut self, byte: T, line: usize) {
        self.data.push(byte.into());
        self.lines.push(line);
    }

    pub(crate) fn make_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        let idx = self.constants.len() - 1;
        if idx > std::u8::MAX as usize {
            panic!("bounds check on constants - idx as u8 will fail");
        } else {
            idx
        }
    }

    #[cfg(feature = "pc-debug")]
    pub fn disassemble(&self, name: &str) {
        use crate::op::Opcode;

        println!(" -- {} --", name);

        let mut prev_line = 0;
        let mut offset = 0;
        while offset < self.data.len() {
            let line = self.get_line_from_index(offset);

            let op = self.data[offset].into();

            print!(
                "{:04} {} {:?}",
                offset,
                if line == prev_line {
                    String::from("   |")
                } else {
                    format!("{:>4}", line)
                },
                op
            );

            offset = match op {
                Opcode::Return => offset + 1,
                Opcode::Constant => {
                    print!(
                        "#{:04} {:?}",
                        self.data[offset + 1],
                        self.constants[self.data[offset + 1] as usize]
                    );
                    offset + 2
                }
                Opcode::Negate => offset + 1,
                Opcode::Add => offset + 1,
                Opcode::Subtract => offset + 1,
                Opcode::Multiply => offset + 1,
                Opcode::Divide => offset + 1,
                Opcode::Nil => offset + 1,
                Opcode::True => offset + 1,
                Opcode::False => offset + 1,
                Opcode::Not => offset + 1,
                Opcode::Equal => offset + 1,
                Opcode::Greater => offset + 1,
                Opcode::Less => offset + 1,
                Opcode::Pop => offset + 1,
                Opcode::DefineGlobal => {
                    print!(
                        "#{:04} {}",
                        self.data[offset + 1],
                        self.constants[self.data[offset + 1] as usize]
                            .clone()
                            .into::<String>(),
                    );
                    offset + 2
                }
                Opcode::GetGlobal => {
                    print!(
                        "#{:04} {}",
                        self.data[offset + 1],
                        self.constants[self.data[offset + 1] as usize]
                            .clone()
                            .into::<String>(),
                    );
                    offset + 2
                }
                Opcode::SetGlobal => {
                    print!(
                        "#{:04} {}",
                        self.data[offset + 1],
                        self.constants[self.data[offset + 1] as usize]
                            .clone()
                            .into::<String>(),
                    );
                    offset + 2
                }
            };
            println!();

            prev_line = line;
        }
    }

    #[cfg(feature = "pc-debug")]
    pub(crate) fn disassemble_instruction(&self, offset: usize) {
        use crate::op::Opcode;

        let line = self.get_line_from_index(offset);

        let op = self.data[offset].into();

        print!("{:04} line {:>6} {:?}", offset, line, op);
        if let Opcode::Constant = op {
            print!(
                "#{:04} {:?}",
                self.data[offset + 1],
                self.constants[self.data[offset + 1] as usize]
            );
        }
        println!();
    }

    pub(crate) fn get_line_from_index(&self, idx: usize) -> usize {
        self.lines[idx]
    }
}