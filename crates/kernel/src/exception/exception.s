// Call the function `\handler` after context saving.
.macro CALL_WITH_CONTEXT handler
__vector_\handler:
    sub     sp, sp, #16 * 17

    // Store all GPRs on the stack.
    stp     x0,  x1,  [sp, #16 * 0]
    stp     x2,  x3,  [sp, #16 * 1]
    stp     x4,  x5,  [sp, #16 * 2]
    stp     x6,  x7,  [sp, #16 * 3]
    stp     x8,  x9,  [sp, #16 * 4]
    stp     x10, x11, [sp, #16 * 5]
    stp     x12, x13, [sp, #16 * 6]
    stp     x14, x15, [sp, #16 * 7]
    stp     x16, x17, [sp, #16 * 8]
    stp     x18, x19, [sp, #16 * 9]
    stp     x20, x21, [sp, #16 * 10]
    stp     x22, x23, [sp, #16 * 11]
    stp     x24, x25, [sp, #16 * 12]
    stp     x26, x27, [sp, #16 * 13]
    stp     x28, x29, [sp, #16 * 14]
    // x30 is lr, it is stored below

    mrs     x1, ELR_EL1     // exception link register
    mrs     x2, SPSR_EL1    // saved program status register
    mrs     x3, ESR_EL1     // exception syndrome register

    stp     lr, x1, [sp, #16 * 15]
    stp     x2, x3, [sp, #16 * 16]

    // Pass `sp` as the first argument to the handler, so it can access the saved context.
    mov     x0, sp

    bl      \handler

    // restore the context after exception handling
    b __exception_restore_context

.size   __vector_\handler, . - __vector_\handler
.type   __vector_\handler, function
.endm

.macro FIQ_SUSPEND
1:
    wfe
    b       1b
.endm

.section .text

// vector table should be aligned to 0x800
.align 11

__exception_vector_start:

// | Exception taken from                                                                            | Synchronous | IRQ   | FIQ   | SError |
// | ----------------------------------------------------------------------------------------------- | ----------- | ----- | ----- | ------ |
// | Current Exception Level with SP_EL0                                                             | 0x000       | 0x080 | 0x100 | 0x180  |
// | Current Exception Level with SP_ELx, x > 0                                                      | 0x200       | 0x280 | 0x300 | 0x380  |
// | Lower Exception Level, where the level immediately lower than the target level is using AArch64 | 0x400       | 0x480 | 0x500 | 0x580  |
// | Lower Exception Level, where the level immediately lower than the target level is using AArch32 | 0x600       | 0x680 | 0x700 | 0x780  |

.org 0x000
    CALL_WITH_CONTEXT current_el0_synchronous
.org 0x080
    CALL_WITH_CONTEXT current_el0_irq
.org 0x100
    FIQ_SUSPEND
.org 0x180
    CALL_WITH_CONTEXT current_el0_serror

.org 0x200
    CALL_WITH_CONTEXT current_elx_synchronous
.org 0x280
    CALL_WITH_CONTEXT current_elx_irq
.org 0x300
    FIQ_SUSPEND
.org 0x380
    CALL_WITH_CONTEXT current_elx_serror

.org 0x400
    CALL_WITH_CONTEXT lower_aarch64_synchronous
.org 0x480
    CALL_WITH_CONTEXT lower_aarch64_irq
.org 0x500
    FIQ_SUSPEND
.org 0x580
    CALL_WITH_CONTEXT lower_aarch64_serror

.org 0x600
    CALL_WITH_CONTEXT lower_aarch32_synchronous
.org 0x680
    CALL_WITH_CONTEXT lower_aarch32_irq
.org 0x700
    FIQ_SUSPEND
.org 0x780
    CALL_WITH_CONTEXT lower_aarch32_serror
.org 0x800  // ensure 0x780-0x800 does not conatin weird stuff

.global __exception_vector_start

__exception_restore_context:
    // top 32 bits of SPSR_EL1 are reserved, so we can just not load it
    ldr     w19,     [sp, #16 * 16]
    ldp     lr, x20, [sp, #16 * 15]

    msr     SPSR_EL1, x19
    msr     ELR_EL1,  x20

    ldp     x0,  x1,  [sp, #16 * 0]
    ldp     x2,  x3,  [sp, #16 * 1]
    ldp     x4,  x5,  [sp, #16 * 2]
    ldp     x6,  x7,  [sp, #16 * 3]
    ldp     x8,  x9,  [sp, #16 * 4]
    ldp     x10, x11, [sp, #16 * 5]
    ldp     x12, x13, [sp, #16 * 6]
    ldp     x14, x15, [sp, #16 * 7]
    ldp     x16, x17, [sp, #16 * 8]
    ldp     x18, x19, [sp, #16 * 9]
    ldp     x20, x21, [sp, #16 * 10]
    ldp     x22, x23, [sp, #16 * 11]
    ldp     x24, x25, [sp, #16 * 12]
    ldp     x26, x27, [sp, #16 * 13]
    ldp     x28, x29, [sp, #16 * 14]

    add     sp, sp, #16 * 17

    eret

.size   __exception_restore_context, . - __exception_restore_context
.type   __exception_restore_context, function
