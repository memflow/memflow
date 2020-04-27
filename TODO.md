- mouse handler + keyboard handler

general:
- remove ida pattern functionality and use something similiar to casualhackers statemachine for patterns
- move unicorn engine functionality in seperate crate

- replace println!() with appropiate log macros
- derive macro for VirtualRead/Write when physical read/write exists
- pin rust version in ci
- key logger / reading
- process emulator
- remove all map_err conversions from flow_core / flow_win32 and others (also in apex and others)
- reduce amount of generic constraints and extend them in implementations (e.g. for virtual reading)
- virt_read_len / virt_read_len32 / virt_read_len64
- move cli into seperate project or inetgrate qemu into this project... < DECIDE >
- move module implementations and helpers (exports, sections, signatures) into core by trait extensions
- Process/Module Iteration -> Streaming Iterator
  - https://docs.rs/streaming-iterator/0.1.5/streaming_iterator/
  - https://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html

mt reading:
- use connection pool and automatically choose a free connection for the reader

virt read/write:
- read into fn phys_read(&mut self, addr: Address, dest: &mut [u8]) -> Result<()>;
- read into generic type T and add a Pod trait for safe types
  -> https://github.com/CasualX/pod/blob/master/zero_copy/src/lib.rs#L39
  -> https://github.com/CasualX/pod/blob/master/zero_copy/src/buffer.rs#L4

processes:
- instead of having process() funcs have a Win32Process::with(mem, win, "ProcessName.exe").unwrap(); or something similiar

cli:
- create neatly wrapped cli with featrues such as:
  - read memory / write memory
  - daemon with multivm/client support + socket support for faster access to a vm
  - try parsing a os -> read processes / parse pe stuff (exports, etc for use with grep)
  - select process -> read modules / parse pe stuff (for use with grep)
- add capstone to disassemble addresses/code
- add sigscanner

va:
- find a better way to encapsulate VatImpl
- add more tests to x64 vtop
- add ptov
- move va into flow-core and use forward declares in core
- could va serve as an entirely seperate crate (and we just provide the trait impls locally?)

win:
- verify each stage and early abort (e.g. see if offsets can be probed from the pdb/struct parser)
- pdb offset to c header dump thing
- have a WindowsOffsets struct which contains all needed offsets, then init it once (e.g. from pdb or from user) and then use it everywhere, have functions to fetch pdb handle and manually init it from pdb

download:
- fork duma and add proper lib stuff (create a struct instead of argopts)

core/arch:
- find a better way to define architecture agnostic type lengths

core/addr:
- tests
- more operator overloads as appropiate

pci/tlp:
- http://xillybus.com/tutorials/pci-express-tlp-pcie-primer-tutorial-guide-1
- create custom parse/generate crate for basic tlps

hex:
- use m4b/hexplay for colored hex output

ffi:
- https://github.com/jimfleming/rust-ffi-complex-types
c -> buck + makefiles + package config + installer
js -> fix compilation + example
python -> add example
more languages ->
move bindings into root folder

cli:
- add a feature to parse / edit xmls via the cli so we can make sure all vms are setup for memflow
  and we can resolve the socket url by vm name
- remote cli (which can access the cli-daemon remotly) with those features as well!

general:
- c bindings
https://www.greyblake.com/blog/2017-08-10-exposing-rust-library-to-c/
- js bindings
https://doc.rust-lang.org/1.2.0/book/rust-inside-other-languages.html

other:
- create a rpm/tlp32 wrapper/hook so u can inject memflow into any regular rpm tool

goblin fixes:
https://github.com/libvmi/libvmi/blob/master/tools/windows-offset-finder/getGUID.cpp#L309

rekall winmem:
- https://github.com/google/rekall/tree/master/tools/windows/winpmem

....

#define first_cpu        QTAILQ_FIRST_RCU(&cpus)
#define CPU_NEXT(cpu)    QTAILQ_NEXT_RCU(cpu, node)
#define CPU_FOREACH(cpu) QTAILQ_FOREACH_RCU(cpu, &cpus, node)
#define CPU_FOREACH_SAFE(cpu, next_cpu) \
    QTAILQ_FOREACH_SAFE_RCU(cpu, &cpus, node, next_cpu)

extern CPUTailQ cpus;

#define QTAILQ_FOREACH_RCU(var, head, field)                            \
    for ((var) = atomic_rcu_read(&(head)->tqh_first);                   \
         (var);                                                         \
         (var) = atomic_rcu_read(&(var)->field.tqe_next))

....

https://github.com/qemu/qemu/blob/33f18cf7dca7741d3647d514040904ce83edd73d/monitor/misc.c#L439
static void hmp_info_registers(Monitor *mon, const QDict *qdict)

cs = mon_get_cpu();
cpu_dump_state(cs, NULL, CPU_DUMP_FPU);

....

void cpu_dump_state(CPUState *cpu, FILE *f, int flags)
{
    CPUClass *cc = CPU_GET_CLASS(cpu);

    if (cc->dump_state) {
        cpu_synchronize_state(cpu);
        cc->dump_state(cpu, f, flags);
    }
}

...

https://github.com/qemu/qemu/blob/95a9457fd44ad97c518858a4e1586a5498f9773c/target/i386/helper.c#L409
void x86_cpu_dump_state(CPUState *cs, FILE *f, int flags)

X86CPU *cpu = X86_CPU(cs);
CPUX86State *env = &cpu->env;
..
env->regs[R_EAX],
env->regs[R_EBX],


2020:
https://github.com/SamuelTulach/efi-memory/
- hooking lib
- hibernation file connector
- pcileech connector
- process simulator
- demo with driver exploit
- demo with uefi
