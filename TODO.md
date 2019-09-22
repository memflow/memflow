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