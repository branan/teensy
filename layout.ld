MEMORY
{
    FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K
    RAM  (rwx) : ORIGIN = 0x1FFF8000, LENGTH = 64K
}

SECTIONS
{
    .text : {
        . = 0;
        KEEP(*(.vectors))
        . = 0x400;
        KEEP(*(.flashconfig*))
        . = ALIGN(4);
        *(.text*)
    } > FLASH = 0xFF

    .rodata : {
        *(.rodata*)
    } > FLASH

    .bss : {
         . = ORIGIN(RAM);
         _bss_start = .;
         *(.bss*)
         _bss_end = .;
    } > RAM

    _stack_top = ORIGIN(RAM) + LENGTH(RAM);

    /DISCARD/ : {
        *(.ARM.*)
    }
}
