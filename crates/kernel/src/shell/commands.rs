use super::ShellCommand;
use crate::{cpio::CpioArchive, driver, exception};
use alloc::boxed::Box;
use small_std::{print, println};

pub struct Hello;

impl ShellCommand for Hello {
    fn name(&self) -> &str {
        "hello"
    }

    fn help(&self) -> &str {
        "print Hello World!"
    }

    fn execute(&self, _: &str) {
        println!("Hello World!");
    }
}

pub struct Reboot;

impl ShellCommand for Reboot {
    fn name(&self) -> &str {
        "reboot"
    }

    fn help(&self) -> &str {
        "reboot the device"
    }

    fn execute(&self, _: &str) {
        driver::watchdog().reset(100);
    }
}

pub struct Info;

impl ShellCommand for Info {
    fn name(&self) -> &str {
        "info"
    }

    fn help(&self) -> &str {
        "print hardware information"
    }

    fn execute(&self, _: &str) {
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

pub struct Ls<'a> {
    cpio: &'a CpioArchive,
}

impl<'a> Ls<'a> {
    pub fn new(cpio: &'a CpioArchive) -> Self {
        Self { cpio }
    }
}

impl<'a> ShellCommand for Ls<'a> {
    fn name(&self) -> &str {
        "ls"
    }

    fn help(&self) -> &str {
        "list files in the initramfs"
    }

    fn execute(&self, _: &str) {
        for file in self.cpio.files() {
            println!("{}", file.filename);
        }
    }
}

pub struct Cat<'a> {
    pub cpio: &'a CpioArchive,
}

impl<'a> Cat<'a> {
    pub fn new(cpio: &'a CpioArchive) -> Self {
        Self { cpio }
    }
}

impl<'a> ShellCommand for Cat<'a> {
    fn name(&self) -> &str {
        "cat"
    }

    fn help(&self) -> &str {
        "cat <file>...\t\tprint content of a file in the initramfs"
    }

    fn execute(&self, args: &str) {
        let mut filenames = args.split_whitespace().peekable();
        if filenames.peek().is_none() {
            println!("Usage: {} <file>...", self.name());
            return;
        }

        filenames.for_each(|filename| {
            match self.cpio.files().find(|f| f.filename == filename) {
                Some(file) => {
                    file.content.iter().for_each(|c| print!("{}", *c as char));
                }
                None => {
                    println!("{}: {}: No such file or directory", self.name(), filename);
                }
            };
        });
    }
}

pub struct Exec<'a> {
    pub cpio: &'a CpioArchive,
}

impl<'a> Exec<'a> {
    pub fn new(cpio: &'a CpioArchive) -> Self {
        Self { cpio }
    }
}

impl<'a> ShellCommand for Exec<'a> {
    fn name(&self) -> &str {
        "exec"
    }

    fn help(&self) -> &str {
        "exec <program>\t\texecute the program in the initramfs"
    }

    fn execute(&self, args: &str) {
        let filename = match args.split_whitespace().next() {
            Some(filename) => filename,
            None => {
                println!("Usage: {} <program>", self.name());
                return;
            }
        };

        let program = match self.cpio.files().find(|f| f.filename == filename) {
            Some(program) => program,
            None => {
                println!("{}: {}: No such file or directory", self.name(), filename);
                return;
            }
        };

        let stack = Box::new([0u8; 0x1000]);
        let stack_end = stack.as_ptr() as u64 + 0x1000;
        unsafe {
            exception::transition_from_el1_to_el0(stack_end, program.content.as_ptr() as u64);
        }
    }
}
