use std::io::{self, Write};

use flow_core::*;
use flow_win32::*;

use pelite::{self, PeView};

pub struct Win32Interface<'a, T>
where
    T: AccessPhysicalMemory + AccessVirtualMemory,
{
    pub mem: &'a mut T,
    pub os: Win32,
    pub offsets: Win32Offsets,

    pub process: Option<Win32Process>,

    pub module: Option<Win32Module>,
}

impl<'a, T> Win32Interface<'a, T>
where
    T: AccessPhysicalMemory + AccessVirtualMemory,
{
    pub fn with(mem: &'a mut T, os: Win32) -> flow_core::Result<Self> {
        let offsets = Win32Offsets::try_with_guid(&os.kernel_guid())?;
        Ok(Self {
            mem,
            os,
            offsets,
            process: None,
            module: None,
        })
    }

    pub fn run(&mut self) -> flow_core::Result<()> {
        let con = unsafe { libc::isatty(0) != 0 };

        let cmds = vec![
            Command::<Self> {
                name: "process",
                description: "",
                func: None,
                subcmds: vec![
                    Command {
                        name: "ls",
                        description: "",
                        func: Some(Self::process_ls),
                        subcmds: Vec::new(),
                    },
                    Command {
                        name: "open",
                        description: "",
                        func: Some(Self::process_open),
                        subcmds: Vec::new(),
                    },
                ],
            },
            Command {
                name: "module",
                description: "",
                func: None,
                subcmds: vec![
                    Command {
                        name: "ls",
                        description: "",
                        func: Some(Self::module_ls),
                        subcmds: Vec::new(),
                    },
                    Command {
                        name: "open",
                        description: "",
                        func: Some(Self::module_open),
                        subcmds: Vec::new(),
                    },
                ],
            },
            Command {
                name: "pe",
                description: "",
                func: None,
                subcmds: vec![
                    Command {
                        name: "exports",
                        description: "",
                        func: Some(Self::pe_exports),
                        subcmds: Vec::new(),
                    },
                    Command {
                        name: "imports",
                        description: "",
                        func: Some(Self::pe_imports),
                        subcmds: Vec::new(),
                    },
                ],
            },
        ];

        loop {
            // If user is at a console, print a nice REPL
            if con {
                print!(">>> ");
                io::stdout().flush().ok();
            }
            // Read input from stdin
            let mut line = String::new();
            if io::stdin().read_line(&mut line).is_err() {
                break;
            }
            // Not sure how to handle ctrl-c events, Rust’s read_line is a bit weird in this regard
            // I basically get an empty string as opposed to a newline when you just press enter.
            if line.is_empty() {
                break;
            }
            // If you press enter without any input, just retry without evaluating.
            let line = line.trim();
            if !line.is_empty() {
                execute_command(self, &cmds, line);

                //println!("<<< {}", line);
                /*
                // TODO: check length
                if tokens.len() > 0 {
                    match tokens[0] {
                        "process" => {
                            if tokens.len() > 1 {
                            match tokens[1] {
                                "ls" => self.process_list(),
                                _ => println!("invalid cmd: '{}'", line),
                            };
                        }
                        }
                        _ => {
                            println!("invalid cmd: '{}'", line);
                        }
                    }
                }
                */
            }
        }
        Ok(())
    }

    fn process_ls(&mut self, _args: Vec<&str>) {
        let eprocs = self.os.eprocess_list(self.mem, &self.offsets).unwrap();
        eprocs
            .iter()
            .map(|eproc| Win32Process::try_with_eprocess(self.mem, &self.os, &self.offsets, *eproc))
            .filter_map(std::result::Result::ok)
            .for_each(|p| println!("{} {}", p.pid(), p.name()));
    }

    fn process_open(&mut self, args: Vec<&str>) {
        if args.is_empty() {
            println!("unable to open process: no process id or process name specified");
            return;
        }

        let procs = Win32Process::try_with_name(self.mem, &self.os, &self.offsets, args[1]);
        match procs {
            Ok(p) => {
                println!("successfully opened process '{}': {:?}", args[1], p);
                self.process = Some(p);
                self.module = None;
            }
            Err(e) => {
                println!("unable to open process '{}': {:?}", args[1], e);
                self.process = None;
                self.module = None;
            }
        }
    }

    fn module_ls(&mut self, _args: Vec<&str>) {
        if self.process.is_none() {
            println!("no process opened. use process open 'name' to open a process");
            return;
        }

        self.process
            .as_ref()
            .unwrap()
            .peb_list(self.mem)
            .unwrap()
            .iter()
            .map(|peb| {
                Win32Module::try_with_peb(
                    self.mem,
                    self.process.as_ref().unwrap(),
                    &self.offsets,
                    *peb,
                )
            })
            .filter_map(std::result::Result::ok)
            .for_each(|module| {
                println!(
                    "{:x} - {:x} + {:x} -> {}",
                    module.base(),
                    module.base(),
                    module.size(),
                    module.name()
                )
            });
    }

    fn module_open(&mut self, args: Vec<&str>) {
        if self.process.is_none() {
            println!("no process opened. use process open 'name' to open a process first");
            return;
        }

        if args.is_empty() {
            println!("unable to open module: module name not specified");
            return;
        }

        let mods = Win32Module::try_with_name(
            self.mem,
            self.process.as_ref().unwrap(),
            &self.offsets,
            args[1],
        );
        match mods {
            Ok(m) => {
                println!("successfully opened module '{}': {:?}", args[1], m);
                self.module = Some(m);
            }
            Err(e) => {
                println!("unable to open module '{}': {:?}", args[1], e);
                self.module = None;
            }
        }
    }

    fn pe_exports(&mut self, _args: Vec<&str>) {
        if self.process.is_none() {
            println!("no process opened. use process open 'name' to open a process");
            return;
        }

        if self.module.is_none() {
            println!("no module opened. use module open 'name' to open a module");
            return;
        }

        let mut virt_mem = self.process.as_ref().unwrap().virt_mem(self.mem);
        let module_buf = virt_mem
            .virt_read_raw(
                self.module.as_ref().unwrap().base(),
                self.module.as_ref().unwrap().size(),
            )
            .unwrap();
        let pe = PeView::from_bytes(&module_buf).unwrap();
        let exports = pe.exports().unwrap();

        exports
            .by()
            .unwrap()
            .names()
            .iter()
            .zip(exports.by().unwrap().functions())
            .for_each(|(&name_rva, function_rva)| {
                let name_it = pe.derva_c_str(name_rva).unwrap().as_ref();
                println!(
                    "{:x} + {:x} -> {}",
                    self.module.as_ref().unwrap().base(),
                    function_rva,
                    std::str::from_utf8(name_it).unwrap()
                );
            });

        /*
        let export_addr = match pe.get_export_by_name("gafAsyncKeyState")? {
            Export::Symbol(s) => kernel_module.base() + Length::from(*s),
            Export::Forward(_) => {
                return Err(flow_win32::Error::new(
                    "export gafAsyncKeyState found but it is forwarded",
                ))
            }
        };
        */
    }

    fn pe_imports(&mut self, _args: Vec<&str>) {}
}

struct Command<'a, T> {
    pub name: &'a str,
    pub description: &'a str,
    pub func: Option<fn(&mut T, Vec<&str>) -> ()>,
    pub subcmds: Vec<Command<'a, T>>,
}

fn execute_command<T>(selfptr: &mut T, cmds: &[Command<T>], line: &str) {
    let tokens = line.split(' ').collect::<Vec<_>>();
    match find_command(selfptr, cmds, tokens) {
        Ok(_) => (),
        Err(e) => println!("error: {:?}", e),
    };
}

fn find_command<'a, T>(
    selfptr: &mut T,
    cmds: &'a [Command<'a, T>],
    input: Vec<&str>,
) -> flow_core::Result<()> {
    for cmd in cmds {
        if input[0] == cmd.name {
            if cmd.func.is_some() {
                (cmd.func.unwrap())(selfptr, input);
                return Ok(());
            } else if input.len() > 1 {
                return find_command(selfptr, &cmd.subcmds, input[1..].to_vec());
            } else {
                return Err(flow_core::Error::new(format!(
                    "sub command not found. valid sub commands: {}",
                    cmd.subcmds
                        .iter()
                        .map(|c| c.name)
                        .collect::<Vec<&str>>()
                        .join(", ")
                )));
            }
        }
    }
    Err(flow_core::Error::new(format!(
        "command not found. valid commands: {}",
        cmds.iter()
            .map(|c| c.name)
            .collect::<Vec<&str>>()
            .join(", ")
    )))
}
