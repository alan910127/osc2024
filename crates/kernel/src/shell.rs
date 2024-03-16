use small_std::{fmt::print::console::console, print, println, string::String};

use crate::driver;

pub struct Shell {
    buf: String,
}

impl Shell {
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    pub fn run_loop(&mut self) -> ! {
        loop {
            self.show_prompt();
            self.read_input();
            self.handle_input();
        }
    }

    fn show_prompt(&self) {
        print!("$ ");
    }

    fn read_input(&mut self) {
        loop {
            let c = console().read_char();

            match c {
                '\r' | '\n' => {
                    println!();
                    return;
                }
                '\x08' | '\x7f' => self.on_backspace(),
                ' '..='~' => self.on_input(c),
                _ => continue,
            }
        }
    }

    fn on_backspace(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        self.buf.pop();
        // go back, write space, then go back again
        print!("\x08 \x08");
    }

    fn on_input(&mut self, c: char) {
        self.buf.push(c);
        print!("{}", c);
    }

    fn handle_input(&mut self) {
        let input = self.buf.as_str();
        let mut args = input.trim().split(" ");
        if let Some(cmd) = args.next() {
            match cmd {
                "help" => self.help(),
                "hello" => self.hello(),
                "reboot" => self.reboot(),
                "info" => self.hardware_info(),
                "" => {}
                _ => println!("{}: command not found", cmd),
            }
        }
        self.buf.clear()
    }

    fn help(&self) {
        println!("help\t: print this help menu");
        println!("hello\t: print Hello World!");
        println!("reboot\t: reboot the device");
        println!("info\t: get hardware informations");
    }

    fn hello(&self) {
        println!("Hello World!");
    }

    fn reboot(&self) {
        driver::watchdog().reset(100);
    }

    fn hardware_info(&self) {
        let revision = driver::mailbox().get_board_revision();
        let memory = driver::mailbox().get_arm_memory();

        match revision {
            Ok(r) => println!("Board revision: {:#x}", r),
            Err(e) => println!("Failed to get board revision: {}", e),
        };
        match memory {
            Ok(m) => {
                println!("ARM Memory base address: {:#x}", m.base_address);
                println!("ARM Memory size: {:#x}", m.size);
            }
            Err(e) => println!("Failed to get memory info: {}", e),
        };
    }
}
