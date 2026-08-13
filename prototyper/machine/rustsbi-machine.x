/*
 * Default image layout for rustsbi-machine.
 *
 * A final image can override __rustsbi_machine_link_start, or replace this
 * script while preserving the exported symbols consumed by the runtime.
 */
OUTPUT_ARCH(riscv)
ENTRY(_start)

PROVIDE(__rustsbi_machine_link_start = 0x80000000);

SECTIONS
{
    . = __rustsbi_machine_link_start;
    __rustsbi_machine_image_start = .;

    .text : ALIGN(16)
    {
        KEEP(*(.text.rustsbi_machine.entry));
        *(.text .text.*)
    }

    .rodata : ALIGN(16)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(16)
    {
        /* Startup atomics must remain initialized while .bss is cleared. */
        KEEP(*(.data.rustsbi_machine.boot));
        *(.data .data.*)
        . = ALIGN(16);
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.*)
        *(.got .got.*)
    }

    .bss (NOLOAD) : ALIGN(16)
    {
        __rustsbi_machine_bss_start = .;
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        __rustsbi_machine_bss_end = .;
    }

    .uninit.rustsbi_machine (NOLOAD) : ALIGN(16)
    {
        KEEP(*(.uninit.rustsbi_machine.stacks));
    }

    . = ALIGN(16);
    __rustsbi_machine_image_end = .;

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame.*)
    }
}

ASSERT((__rustsbi_machine_bss_start % 16) == 0,
       "rustsbi-machine: .bss start must be 16-byte aligned");
ASSERT((__rustsbi_machine_bss_end % 16) == 0,
       "rustsbi-machine: .bss end must be 16-byte aligned");
