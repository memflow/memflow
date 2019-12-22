/*
process::process_emulator pe( this->m_process );
process::process_emulator::process_emulator_context ctx;
ctx.address = ci;
ctx.set_parameters = [ & ]( ) {
    pe.push<uint32_t>( 0 );
    pe.push_ptr( ( void * )iface_version.c_str( ), iface_version.size( ) + 1 );
    pe.push<uint32_t>( 0 ); // push ret address from the call opcode
};
ctx.get_result = [ & ]( ) {
    uint32_t eax = pe.reg_read<uint32_t>( UC_X86_REG_EAX );
    this->m_interfaces[ iface ] = eax;
};
pe.exec( ctx );
*/

/*
Emulator::with(process) -> Emulator
    .emulation(addr) // -> EmulationContext
    .execute(|e| { /* pre hook */ })
    .run() -> EmulationResult ?
    .extract(|e| { /* post hook */ }) -> T
    .ok_or_else(|| "some error msg")
*/

use crate::address::Address;
use crate::arch::{ArchitectureTrait, InstructionSet};
use crate::process::ProcessTrait;
use crate::{Error, Result};

use unicorn::CpuX86;

// TODO: module iteration should be relocated to flow_core somehow
pub struct Emulator<T: ProcessTrait + ArchitectureTrait> {
    process: T,
    // set_params_fn
    // get_result_fn
    // ...
}

pub struct EmulationContext {}

pub struct EmulationResult {
    // ...
}

impl<T: ProcessTrait + ArchitectureTrait> Emulator<T> {
    pub fn with(process: T) -> Self {
        Self { process }
    }

    // TODO: architecture agnostic operation modes!
    pub fn emulation(&mut self, _addr: Address) -> Result<EmulationContext> {
        // 1 - find module containing addr in process
        //let module = self.process.containing_module(addr)?;

        // TODO: unicorn::arch_supported()
        // uc_open()
        let arch = self.process.arch()?;
        let _emu = match arch.instruction_set {
            InstructionSet::X64 => CpuX86::new(unicorn::Mode::MODE_64).map_err(Error::from),
            InstructionSet::X86 => CpuX86::new(unicorn::Mode::MODE_32).map_err(Error::from),
            // TODO: add more
            _ => Err(Error::new("unsupported process architecture")),
        }?;

        // map_from_process64
        // setup rsp/esp
        // mem_map 0->0x2000000 UC_PROT_READ | UC_PROT_WRITE
        // x86 uses 0->0x20000
        // setup hooks
        // uc_emu_start()

        /*
        emu.mem_map
        emu.reg_Write_i32
        emu.start
        */
        Ok(EmulationContext {})
    }
}

/*
process_emulator::exec(process_emulator_context &ctx) {
    if (this->m_process->is_x64()) {
        return this->exec64(ctx);
    } else {
        printf("process_emulator(): 32bit execution is not implemented yet\n");
        return -1;
    }
}
*/

impl EmulationContext {
    pub fn run(&mut self) -> Result<EmulationResult> {
        // setup hooks

        // setup stack

        // run

        // check error

        Ok(EmulationResult {})
    }
}
