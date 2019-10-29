general:
- replace println!() with appropiate log macros

va:
- find a better way to encapsulate VatImpl
- add more tests to x64 vtop
- add ptov
- move va into flow-core and use forward declares in core
- could va serve as an entirely seperate crate (and we just provide the trait impls locally?)

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

qemu:
- add option to access rpc via unix/udp/tcp

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
