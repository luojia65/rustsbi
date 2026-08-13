    .section .text.rustsbi_machine.entry, "ax"
    .globl _start
    .type _start, @function
    .align 2
_start:
    /* Start from a known interrupt state before a trap vector exists. */
    csrw mie, zero
    li t0, 8
    csrc mstatus, t0

    /* Rust may address small data through gp. Disable relaxation for this load. */
    .option push
    .option norelax
    lla gp, __global_pointer$
    .option pop

    /* Architectural mhartid is authoritative; preserve the incoming a1. */
    csrr a0, mhartid

    /* Claim a dense stack slot without assuming dense hart IDs. */
    li t0, 1
    lla t1, {next_stack}
    .if {xlen} == 64
        amoadd.d.aq a2, t0, (t1)
    .else
        amoadd.w.aq a2, t0, (t1)
    .endif

    li t0, {max_harts}
    bgeu a2, t0, .Lrustsbi_machine_halt

    lla t0, {boot_stacks}
    addi t1, a2, 1
    slli t1, t1, {stack_shift}
    add sp, t0, t1
    andi sp, sp, -16

    /*
     * Slot zero owns cold initialization. Keep all other harts in assembly:
     * entering Rust before BSS has its language-level initial value would make
     * accesses to Rust statics and linker-boundary pointer arithmetic unsound.
     */
    bnez a2, .Lrustsbi_machine_wait_for_bss

    lla t0, __rustsbi_machine_bss_start
    lla t1, __rustsbi_machine_bss_end
.Lrustsbi_machine_clear_bss:
    bgeu t0, t1, .Lrustsbi_machine_bss_ready
    .if {xlen} == 64
        sd zero, 0(t0)
        addi t0, t0, 8
    .else
        sw zero, 0(t0)
        addi t0, t0, 4
    .endif
    j .Lrustsbi_machine_clear_bss

.Lrustsbi_machine_bss_ready:
    /* Publish all BSS stores with release semantics. */
    li t0, {boot_ready}
    lla t1, {boot_state}
    .if {xlen} == 64
        amoswap.d.aqrl zero, t0, (t1)
    .else
        amoswap.w.aqrl zero, t0, (t1)
    .endif
    j .Lrustsbi_machine_enter_rust

.Lrustsbi_machine_wait_for_bss:
    /* Acquire the cold hart's publication before any Rust code can execute. */
    lla t0, {boot_state}
.Lrustsbi_machine_wait_for_bss_loop:
    .if {xlen} == 64
        amoadd.d.aq t1, zero, (t0)
    .else
        amoadd.w.aq t1, zero, (t0)
    .endif
    beqz t1, .Lrustsbi_machine_wait_for_bss_loop

.Lrustsbi_machine_enter_rust:
    /* No caller exists; a tail call also prevents accidental return. */
    tail {rust_entry}

.Lrustsbi_machine_halt:
    wfi
    j .Lrustsbi_machine_halt
    .size _start, . - _start
