SECTIONS
{
    .text :
    {
        KEEP(*(.text.main))
        *(.text*)
    }

    .rodata : ALIGN(8)
    {
        *(.rodata*)
    }

    .data :
    {
        *(.data*)
    }

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive;
    }

    /DISCARD/ :
    {
        *(.comment*)
    }
}
