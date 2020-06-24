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
    ret     = 0x13,
    // reserve some space for all the other operators
    /// discard top of stack
    pop     = 0x60,
    /// load from local variable
    /// loadl <stack_index>
    ///     -> <val>
    iloadl  = 0x72,
    uloadl  = 0x73,
    dloadl  = 0x74,
    rloadl  = 0x75,
    /// store into local variable
    /// storel <stack_index>
    /// <val> -> <val>
    istorel = 0x76,
    ustorel = 0x77,
    dstorel = 0x78,
    rstorel = 0x79,
    /// alloc new array of <type> (array_size from stack)
    newarr  = 0x80,
    /// load from array
    /// <arrayref> <index> <val> ->
    iaload  = 0x82,
    uaload  = 0x83,
    daload  = 0x84,
    raload  = 0x85,
    /// <arrayref> <index> -> <val>
    iastore = 0x86,
    uastore = 0x87,
    dastore = 0x88,
    rastore = 0x89,
}
