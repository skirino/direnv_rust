use std::io;
use ansi_term::Colour;

pub fn print_shell_command_without_log(k: &str, o: &Option<String>) -> () {
    println!("{}", make_shell_command(k, o))
}

pub fn print_shell_command_with_log(k: &str, o: &Option<String>) -> () {
    let s = make_shell_command(k, o);
    println!("{}", s);
    log_green(&("$ ".to_string() + &s))
}

fn make_shell_command(k: &str, o: &Option<String>) -> String {
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
    let s = "direnv_rust: ".to_string() + msg + "\n";
    color.paint(s.as_bytes()).write_to(&mut io::stderr()).unwrap()
}
