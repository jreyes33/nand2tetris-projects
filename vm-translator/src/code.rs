use crate::parser::Command::{self, *};
use crate::parser::Segment::{self, *};

macro_rules! hasm {
    ($($line:expr),* $(,)?) => {
        concat!($($line, '\n',)*)
    };
}

macro_rules! take {
    (1) => {
        hasm!("@SP", "AM=M-1", "D=M")
    };
    (2) => {
        hasm!(take!(1), "@SP", "A=M-1")
    };
}

macro_rules! push_d {
    () => {
        hasm!("@SP", "A=M", "M=D", "@SP", "M=M+1")
    };
}

pub fn translate(commands: &[Option<Command>], static_prefix: &str) -> String {
    let mut translator = Translator::default();
    commands
        .iter()
        .flatten()
        .map(|c| translator.translate(c, static_prefix))
        .collect()
}

pub fn boot() -> String {
    // Set SP to 256 and call Sys.init.
    hasm!("@256", "D=A", "@SP", "M=D").to_string() + &call("Sys.init", "BOOT", 0, 0)
}

#[derive(Default)]
struct Translator {
    cond_counter: u16,
    call_counter: u16,
}

impl Translator {
    fn translate(&mut self, command: &Command, static_prefix: &str) -> String {
        match command {
            Pop(segment, i) => pop(*segment, *i, static_prefix),
            Push(segment, i) => push(*segment, *i, static_prefix),
            Eq | Gt | Lt => {
                self.cond_counter += 1;
                conditional(command, static_prefix, self.cond_counter)
            }
            Add | Sub | Neg | And | Or | Not => arithmetic(command).to_string(),
            Label(ident) => label(ident),
            Goto(ident) => format!(hasm!("@{ident}", "0;JMP"), ident = ident),
            IfGoto(ident) => format!(hasm!(take!(1), "@{ident}", "D;JNE"), ident = ident),
            Call(ident, i) => {
                self.call_counter += 1;
                call(ident, static_prefix, *i, self.call_counter)
            }
            Function(ident, i) => function(ident, *i),
            Return => return_().to_string(),
        }
    }
}

fn label(name: &str) -> String {
    format!(hasm!("({name})"), name = name)
}

fn call(function_name: &str, static_prefix: &str, args_count: u16, counter: u16) -> String {
    format!(
        hasm!(
            "@CALL_RET_{static_prefix}_{counter}",
            "D=A",
            push_d!(), // save return address
            "@LCL",
            "D=M",
            push_d!(), // save LCL
            "@ARG",
            "D=M",
            push_d!(), // save ARG
            "@THIS",
            "D=M",
            push_d!(), // save THIS
            "@THAT",
            "D=M",
            push_d!(), // save THAT
            "@SP",
            "D=M",
            "@5",
            "D=D-A",
            "@{args_count}",
            "D=D-A",
            "@ARG",
            "M=D", // reposition ARG
            "@SP",
            "D=M",
            "@LCL",
            "M=D", // reposition LCL
            "@{function_name}",
            "0;JMP", // jump to function
            "(CALL_RET_{static_prefix}_{counter})",
        ),
        static_prefix = static_prefix,
        counter = counter,
        args_count = args_count,
        function_name = function_name,
    )
}

fn function(name: &str, locals_count: u16) -> String {
    let mut code = label(name);
    for _ in 0..locals_count {
        code += hasm!("@SP", "AM=M+1", "A=A-1", "M=0");
    }
    code
}

fn return_() -> &'static str {
    hasm!(
        "@LCL",
        "D=M",
        "@R14",
        "M=D", // save endframe to R14
        "@5",
        "A=D-A",
        "D=M",
        "@R15",
        "M=D", // save retaddr to R15
        take!(1),
        "@ARG",
        "A=M",
        "M=D", // reposition return value
        "D=A",
        "@SP",
        "M=D+1", // reposition SP
        "@R14",
        "A=M-1",
        "D=M",
        "@THAT",
        "M=D", // restore THAT
        "@R14",
        "D=M-1",
        "A=D-1",
        "D=M",
        "@THIS",
        "M=D", // restore THIS
        "@R14",
        "D=M-1",
        "D=D-1",
        "A=D-1",
        "D=M",
        "@ARG",
        "M=D", // restore ARG
        "@R14",
        "D=M-1",
        "D=D-1",
        "D=D-1",
        "A=D-1",
        "D=M",
        "@LCL",
        "M=D", // restore LCL
        "@R15",
        "A=M",
        "0;JMP", // jump to retaddr
    )
}

fn conditional(command: &Command, static_prefix: &str, counter: u16) -> String {
    let jump = match command {
        Eq => "JEQ",
        Gt => "JGT",
        Lt => "JLT",
        _ => unreachable!("should not be called with any other command"),
    };
    format!(
        hasm!(
            take!(2),
            "D=M-D",
            "M=-1",
            "@COND_{static_prefix}_{counter}",
            "D;{jump}",
            "@SP",
            "A=M-1",
            "M=0",
            "(COND_{static_prefix}_{counter})",
        ),
        static_prefix = static_prefix,
        counter = counter,
        jump = jump,
    )
}

fn arithmetic(command: &Command) -> &'static str {
    match command {
        Add => hasm!(take!(2), "M=D+M"),
        Sub => hasm!(take!(2), "M=M-D"),
        Neg => hasm!("@SP", "A=M-1", "M=-M"),
        And => hasm!(take!(2), "M=D&M"),
        Or => hasm!(take!(2), "M=D|M"),
        Not => hasm!("@SP", "A=M-1", "M=!M"),
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
            "M=D",    // @R13 has the target address where we want to store the popped value.
            take!(1), // D has the popped value.
            "@R13",
            "A=M",
            "M=D", // Finally, the value is stored in the target address.
        ),
        i = i,
        base_addr = base_addr,
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
            "M=D",    // @R13 has the target address where we want to store the popped value.
            take!(1), // D has the popped value.
            "@R13",
            "A=M",
            "M=D", // Finally, the value is stored in the target address.
        ),
        i = i,
    )
}

fn pop_static(prefix: &str, i: u16) -> String {
    format!(
        hasm!(take!(1), "@{prefix}.{i}", "M=D"),
        prefix = prefix,
        i = i,
    )
}

fn pop_pointer(i: u16) -> String {
    format!(hasm!(take!(1), "@{addr}", "M=D"), addr = point_to(i))
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
    format!(hasm!("@{i}", "D=A", push_d!()), i = i)
}

fn push_i(base_addr: &str, i: u16) -> String {
    format!(
        hasm!("@{i}", "D=A", "@{base_addr}", "AD=D+M", "D=M", push_d!()),
        i = i,
        base_addr = base_addr,
    )
}

fn push_temp(i: u16) -> String {
    format!(
        hasm!("@{i}", "D=A", "@5", "AD=D+A", "D=M", push_d!()),
        i = i,
    )
}

fn push_static(prefix: &str, i: u16) -> String {
    format!(
        hasm!("@{prefix}.{i}", "D=M", push_d!()),
        prefix = prefix,
        i = i,
    )
}

fn push_pointer(i: u16) -> String {
    format!(hasm!("@{addr}", "D=M", push_d!()), addr = point_to(i))
}

fn point_to(i: u16) -> &'static str {
    match i {
        0 => "THIS",
        1 => "THAT",
        _ => unimplemented!("pointer segment only supports 0 and 1"),
    }
}
