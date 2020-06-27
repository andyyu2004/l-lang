use num_enum::TryFromPrimitive;

/// Opcode
/// 3 numeric types
/// i = integer = i64; u = unsigned = u64; d = double = f64
#[allow(non_camel_case_types)]
#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Op {
    ///     ->
    nop     = 0x00,
    /// iconst <val>
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
    /// <val> -> []
    iret    = 0x10,
    uret    = 0x11,
    dret    = 0x12,
    rret    = 0x13,
    ret     = 0x14,
    unit    = 0x15,
    // reserve some space for all the other operators
    /// discard top of stack
    pop     = 0x60,
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
    /// load from upvalue
    /// loadu <index> (index into upvalue array)
    ///     -> <val>
    iloadu  = 0x78,
    uloadu  = 0x79,
    dloadu  = 0x7A,
    rloadu  = 0x7B,
    /// store into upvalue
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

    /// invoke <argc>
    /// <f> <arg_0>...<arg_argc> -> <f> <arg_0> ... <arg_argc>
    invoke  = 0xA0,
    /// pushes a closure onto the stack
    /// clsr (<in_enclosing> <index>)+
    /// <f_idx> -> <closure>
    clsr    = 0xA1,
}
