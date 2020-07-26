use num_enum::TryFromPrimitive;

/// Opcode
/// 3 numeric types
/// i = integer = i64; u = unsigned = u64; d = double = f64
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Op {
    ///     ->
    ///
    nop     = 0x00,
    /// const <val>
    ///     -> i64
    iconst  = 0x01,
    uconst  = 0x02,
    dconst  = 0x03,
    /// <val> <val> -> <val>
    iadd    = 0x04,
    uadd    = 0x05,
    dadd    = 0x06,
    isub    = 0x07,
    usub    = 0x08,
    dsub    = 0x09,
    imul    = 0x0A,
    umul    = 0x0B,
    dmul    = 0x0C,
    idiv    = 0x0D,
    udiv    = 0x0E,
    ddiv    = 0x0F,
    dcmplt  = 0x10,
    dcmpgt  = 0x11,
    /// unconditional jmp
    jmp     = 0x20,
    /// jump if false
    jmpf    = 0x21,
    /// jmp if true
    jmpt    = 0x22,
    jmpeq   = 0x23,
    jmpneq  = 0x24,
    /// <val> -> []
    iret    = 0x30,
    uret    = 0x31,
    dret    = 0x32,
    rret    = 0x33,
    ret     = 0x34,
    unit    = 0x35,

    /// discard top of stack
    pop     = 0x60,
    dup     = 0x61,
    /// load from local variable
    /// loadl <stack_index>
    ///     -> <val>
    iloadl  = 0x70,
    uloadl  = 0x71,
    dloadl  = 0x72,
    rloadl  = 0x73,
    /// store into local variable
    /// storel <stack_index>
    /// <val> -> <val>
    istorel = 0x74,
    ustorel = 0x75,
    dstorel = 0x76,
    rstorel = 0x77,
    /// load from upvar
    /// loadu <index> (index into upvalue array)
    ///     -> <val>
    iloadu  = 0x78,
    uloadu  = 0x79,
    dloadu  = 0x7A,
    rloadu  = 0x7B,
    /// store into upvar
    istoreu = 0x7C,
    ustoreu = 0x7D,
    dstoreu = 0x7E,
    rstoreu = 0x7F,
    /// load from constant pool
    /// ldx <index>
    ///     -> <constant>
    ldc     = 0x80,
    /// alloc new array of <type> (array_size from stack)
    newarr  = 0x90,
    /// load from array
    /// <arrayref> <index> <val> ->
    iaload  = 0x92,
    uaload  = 0x93,
    daload  = 0x94,
    raload  = 0x95,
    /// <arrayref> <index> -> <val>
    iastore = 0x96,
    uastore = 0x97,
    dastore = 0x98,
    rastore = 0x99,

    /// call <argc>
    /// <f> <arg_0>...<arg_argc> -> <f> <arg_0> ... <arg_argc>
    call    = 0xA0,
    /// pushes a closure onto the stack
    /// clsr <const_idx> (<in_enclosing> <index>)+
    /// -> <closure>
    clsr    = 0xA1,
    /// popscope <n>
    /// pop <n> local variables from stack while retaining value of the block
    /// example:
    /// [0,1,2,3,4] -> popscp 2 -> [0,1,4]
    popscp  = 0xA4,

    mktup   = 0xC0,
    mklst   = 0xC1,
    mkmap   = 0xC2,
}
