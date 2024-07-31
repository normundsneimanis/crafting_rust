use std::fmt::{Display, Formatter};

pub enum OpCode {
    OpConstant,
    OpReturn,
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::OpConstant,
            1 => OpCode::OpReturn,
            2 => OpCode::OpNegate,
            3 => OpCode::OpAdd,
            4 => OpCode::OpSubtract,
            5 => OpCode::OpMultiply,
            6 => OpCode::OpDivide,
            _ => {eprintln!("Unknown opcode conversion attempt: {}", value); std::process::exit(1)}
        }
    }
}

macro_rules! binary_op {
    ($self:ident, $op:tt) => {{
        let b = $self.pop();
        let a = $self.pop();
        match (a, b) {
            (VmValue::Double(a_), VmValue::Double(b_)) => $self.push(VmValue::Double(a_ $op b_)),
            // _ => panic!()
        }
    }};
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::OpReturn => f.write_str("OpReturn"),
            OpCode::OpConstant => f.write_str("OpConstant"),
            OpCode::OpNegate => f.write_str("OpNegate"),
            OpCode::OpAdd => f.write_str("OpAdd"),
            OpCode::OpSubtract => f.write_str("OpSubtract"),
            OpCode::OpMultiply => f.write_str("OpMultiply"),
            OpCode::OpDivide => f.write_str("OpDivide"),
        }
    }
}

pub struct Chunk {
    count: usize,
    capacity: usize,
    code: Box<[u8]>,
    value_array: ValueArray,
    src_location: Box<[SrcLocation]>
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    debug_disassemble_instructions: bool,
    stack: Box<[VmValue]>,
    stack_top: usize
}


impl Default for Vm {
    fn default() -> Self {
        Vm{chunk: Chunk::default(),
            ip: 0,
            debug_disassemble_instructions: false,
            stack: vec![VmValue::Double(0.0); 256].into_boxed_slice(),
            stack_top: 0,
        }
    }
}

impl Vm {
    // fn reset(&mut self) {
    //     self.ip = 0;
    //     self.stack_top = 0;
    // }

    fn push(&mut self, vm_value: VmValue) {
        self.stack[self.stack_top] = vm_value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> VmValue {
        self.stack_top -= 1;
        return self.stack[self.stack_top].clone();
    }

    pub fn enable_debug(&mut self) {
        self.debug_disassemble_instructions = true;
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0usize;
        return self.run();
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if self.debug_disassemble_instructions {
                print!("Stack: ");
                for i in 0..self.stack_top {
                    print!("[{}] ", self.stack[i])
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }
            let instruction = OpCode::from(self.read_byte());
            match instruction {
                OpCode::OpReturn => {
                    let val = &self.pop();
                    self.chunk.print_value(val);
                    return InterpretResult::InterpretOk;
                },
                OpCode::OpConstant => {
                    let value = self.read_constant();
                    self.push(value.clone());
                    println!("{}", value);
                }
                OpCode::OpNegate => {
                    let tmp = self.pop();
                    match tmp {
                        VmValue::Double(f) => self.push(VmValue::Double(-f)),
                    }
                }
                OpCode::OpAdd => binary_op!(self, +),
                OpCode::OpSubtract => binary_op!(self, -),
                OpCode::OpMultiply => binary_op!(self, *),
                OpCode::OpDivide => binary_op!(self, /),
            }
        }
    }

    fn read_constant(&mut self) -> VmValue {
        self.chunk.value_array.values[self.read_byte() as usize].clone()
    }

    fn read_byte(&mut self) -> u8 {
        let ret = self.chunk.code[self.ip];
        self.ip += 1;
        ret
    }
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk{count: 0, capacity: 0,
            code: vec![].into_boxed_slice(),
            value_array: ValueArray::default(),
            src_location: vec![].into_boxed_slice(),
        }
    }
}

impl Chunk {
    pub fn write_chunk(&mut self, byte: u8, src_location: SrcLocation) {
        if self.capacity < self.count + 1 {
            self.capacity = if self.capacity < 8 { 8 } else { self.capacity * 2 };
            let mut code = vec![0u8; self.capacity].into_boxed_slice();
            self.code.iter().enumerate().for_each(|(n, e)| code[n] = *e);
            self.code = code;
            let mut src_location = vec![SrcLocation{line: 0, col: 0}; self.capacity].into_boxed_slice();
            self.src_location.iter().enumerate().for_each(|(n, e)| src_location[n] = (*e).clone());
            self.src_location = src_location;

        }
        self.code[self.count] = byte;
        self.src_location[self.count] = src_location;
        self.count += 1;
    }

    pub fn add_constant(&mut self, value: f64) -> u8 {
        let vm_value = VmValue::Double(value);
        self.value_array.write_value(vm_value);
        (self.value_array.count - 1) as u8
    }

    pub fn disassemble(&self, name: &str) {
        println!("Chunk {}: ", name);
        let mut offset = 0usize;
        loop {
            offset = self.disassemble_instruction(offset);
            if offset >= self.count {
                break;
            }
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("\t{:04} ", offset);

        if offset > 0 && self.src_location[offset] == self.src_location[offset - 1] {
            print!("   | ");
        } else {
            print!("{} ", self.src_location[offset]);
        }

        let op = OpCode::from(self.code[offset]);
        return match op {
            OpCode::OpReturn => self.simple_instruction(op, offset),
            OpCode::OpConstant => self.constant_instruction(op, offset),
            OpCode::OpNegate => self.simple_instruction(op, offset),
            OpCode::OpAdd => self.simple_instruction(op, offset),
            OpCode::OpSubtract => self.simple_instruction(op, offset),
            OpCode::OpMultiply => self.simple_instruction(op, offset),
            OpCode::OpDivide => self.simple_instruction(op, offset),
            // _ => { println!("Unknown opcode: {}", op); offset + 1 }
        }
    }

    fn constant_instruction(&self, op: OpCode, offset: usize) -> usize {
        let constant = self.code[offset + 1] as usize;
        print!("{:-16} {:04} '", op, constant);
        self.print_value(&self.value_array.values[constant]);
        offset + 2
    }

    fn print_value(&self, vm_value: &VmValue) {
        println!("{}'", vm_value);
    }

    fn simple_instruction(&self, op: OpCode, offset: usize) -> usize {
        println!("{}", op);
        offset + 1
    }
}

#[derive(Clone)]
pub enum VmValue {
    Double(f64)
}


impl Display for VmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VmValue::Double(d) => f.write_str(d.to_string().as_str()),
        }
    }
}

struct ValueArray {
    count: usize,
    capacity: usize,
    values: Box<[VmValue]>,
}

impl Default for ValueArray {
    fn default() -> Self {
        ValueArray{count: 0, capacity: 0, values: vec![].into_boxed_slice()}
    }
}

impl ValueArray {
    fn write_value(&mut self, value: VmValue) {
        if self.capacity < self.count + 1 {
            self.capacity = if self.capacity < 8 { 8 } else { self.capacity * 2 };
            let mut values = vec![VmValue::Double(0f64); self.capacity].into_boxed_slice();
            self.values.iter().enumerate().for_each(|(n, e)| values[n] = (*e).clone());
            self.values = values;
        }
        self.values[self.count] = value;
        self.count += 1;
    }
}

#[derive(Clone, PartialEq)]
pub struct SrcLocation {
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl Display for SrcLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("line: {:4} col: {:3}", self.line, self.col).as_str())
    }
}