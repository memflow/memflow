general:
- replace println!() with appropiate log macros

va:
- find a better way to encapsulate VatImpl
- add more tests to x64 vtop
- add ptov

core/arch:
- find a better way to define architecture agnostic type lengths

core/addr:
- tests
- more operator overloads as appropiate

hex:
- use m4b/hexplay for colored hex output

fetching pdbs:
- https://github.com/m4b/goblin/blob/05c2fe9609a6d1dfc4622b59a0a5d1a9702c39be/src/pe/debug.rs#L26
- rekall fetch_pdb <PDB filename> <GUID>
- rekall parse_pdb <PDB filename> > rekall-profile.json
- https://github.com/google/rekall/blob/a82349758fdc15274501cf41ff5b4bc913698494/rekall-core/rekall/plugins/tools/mspdb.py#L119
- https://github.com/m4b/goblin/pull/183/commits/84b2c37e835e621c549fdae857eecc0fd9ef31d8#diff-d0d67e7a3a3d00cb0e6d863395e0096bR833
- https://github.com/willglynn/pdb

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