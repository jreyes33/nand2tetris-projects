use crate::parser::Command::{self, *};
use crate::parser::Segment::{self, *};

macro_rules! hasm {
    ($($line:expr),* $(,)?) => {
        concat!($($line, '\n',)*)
    };
}

macro_rules! take {
    (1) => {
        hasm!("@SP", "AM=M-1")
    };
    (2) => {
        hasm!(take!(1), "D=M", "@SP", "A=M-1")
    };
}

pub fn translate(command: &Command, static_prefix: &str) -> String {
    match command {
        Pop(segment, i) => pop(*segment, *i, static_prefix),
        Push(segment, i) => push(*segment, *i, static_prefix),
        _ => arithmetic(command).to_string(),
    }
}

fn arithmetic(command: &Command) -> &'static str {
    match command {
        Add => hasm!(take!(2), "M=D+M"),
        Sub => hasm!(take!(2), "M=M-D"),
        Neg => hasm!(take!(1), "M=-M"),
        Eq => todo!(),
        Gt => todo!(),
        Lt => todo!(),
        And => hasm!(take!(2), "M=D&M"),
        Or => hasm!(take!(2), "M=D|M"),
        Not => hasm!(take!(1), "M=!M"),
        _ => unreachable!("should not be called with any other command"),
    }
}

fn pop(segment: Segment, i: u16, static_prefix: &str) -> String {
    match segment {
        Local => pop_i("LCL", i),
        Argument => pop_i("ARG", i),
        This => pop_i("THIS", i),
        That => pop_i("THAT", i),
        Constant => unimplemented!("`pop constant i` doesn't make sense"),
        Static => pop_static(static_prefix, i),
        Pointer => pop_pointer(i),
        Temp => pop_temp(i),
    }
}

fn pop_i(base_addr: &str, i: u16) -> String {
    format!(
        hasm!(
            "@{i}",
            "D=A",
            "@{base_addr}",
            "D=D+M",
            "@R13",
            "M=D", // @R13 has the target address where we want to store the popped value.
            take!(1),
            "D=M", // D has the popped value.
            "@R13",
            "A=M",
            "M=D", // Finally, the value is stored in the target address.
        ),
        i = i,
        base_addr = base_addr
    )
}

fn pop_temp(i: u16) -> String {
    format!(
        hasm!(
            "@{i}",
            "D=A",
            "@5",
            "D=D+A",
            "@R13",
            "M=D", // @R13 has the target address where we want to store the popped value.
            take!(1),
            "D=M", // D has the popped value.
            "@R13",
            "A=M",
            "M=D", // Finally, the value is stored in the target address.
        ),
        i = i,
    )
}

fn pop_static(prefix: &str, i: u16) -> String {
    format!(
        hasm!(take!(1), "D=M", "@{prefix}.{i}", "M=D"),
        prefix = prefix,
        i = i,
    )
}

fn pop_pointer(i: u16) -> String {
    format!(hasm!(take!(1), "D=M", "@{addr}", "M=D"), addr = point_to(i))
}

fn push(segment: Segment, i: u16, static_prefix: &str) -> String {
    match segment {
        Local => push_i("LCL", i),
        Argument => push_i("ARG", i),
        This => push_i("THIS", i),
        That => push_i("THAT", i),
        Constant => push_constant(i),
        Static => push_static(static_prefix, i),
        Pointer => push_pointer(i),
        Temp => push_temp(i),
    }
}

fn push_constant(i: u16) -> String {
    format!(
        hasm!("@{i}", "D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"),
        i = i
    )
}

fn push_i(base_addr: &str, i: u16) -> String {
    format!(
        hasm!(
            "@{i}",
            "D=A",
            "@{base_addr}",
            "D=D+M",
            "A=D",
            "D=M",
            "@SP",
            "A=M",
            "M=D",
            "@SP",
            "M=M+1"
        ),
        i = i,
        base_addr = base_addr
    )
}

fn push_temp(i: u16) -> String {
    format!(
        hasm!("@{i}", "D=A", "@5", "D=D+A", "A=D", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"),
        i = i,
    )
}

fn push_static(prefix: &str, i: u16) -> String {
    format!(
        hasm!("@{prefix}.{i}", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"),
        prefix = prefix,
        i = i
    )
}

fn push_pointer(i: u16) -> String {
    format!(
        hasm!("@{addr}", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"),
        addr = point_to(i)
    )
}

fn point_to(i: u16) -> &'static str {
    match i {
        0 => "THIS",
        1 => "THAT",
        _ => unimplemented!("pointer segment only supports 0 and 1"),
    }
}
