use num_enum::TryFromPrimitive;

/// Opcode
/// 3 numeric types
/// i = integer = i64; u = unsigned = u64; d = double = f64
#[allow(non_camel_case_types)]
#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Op {
    /// nop
    nop    = 0x00,
    /// iconst <i64>
    iconst = 0x01,
    /// uconst <u64>
    uconst = 0x02,
    /// dconst <f64>
    dconst = 0x03,
    /// iadd
    iadd   = 0x04,
    /// uadd
    uadd   = 0x05,
    /// dadd
    dadd   = 0x06,
    /// isub
    isub   = 0x07,
    /// usub
    usub   = 0x08,
    /// dsub
    dsub   = 0x09,
    /// imul
    imul   = 0x0A,
    /// umul
    umul   = 0x0B,
    /// dmul
    dmul   = 0x0C,
    /// idiv
    idiv   = 0x0D,
    /// udiv
    udiv   = 0x0E,
    /// ddiv
    ddiv   = 0x0F,
    /// iret
    iret   = 0x10,
    /// uret
    uret   = 0x11,
    /// dret
    dret   = 0x12,
}
