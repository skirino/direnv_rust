use std::io;
use ansi_term::Colour;

pub fn print_instruction_without_log(k: &str, o: &Option<String>) -> () {
    println!("{}", make_instruction(k, o))
}

pub fn print_instruction_with_log(k: &str, o: &Option<String>) -> () {
    let s = make_instruction(k, o);
    println!("{}", s);
    log_green(&("direnv_rust: $ ".to_string() + &s + "\n"))
}

fn make_instruction(k: &str, o: &Option<String>) -> String {
    match *o {
        Some(ref v) => format!("export {}='{}'", k, v),
        None        => format!("unset {}", k),
    }
}

pub fn log_green(msg: &str) -> () {
    log(Colour::Green, msg)
}

pub fn log_red(msg: &str) -> () {
    log(Colour::Red, msg)
}

fn log(color: Colour, msg: &str) -> () {
    color.paint(msg.as_bytes()).write_to(&mut io::stderr()).unwrap()
}
