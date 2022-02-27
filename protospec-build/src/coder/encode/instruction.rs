use super::*;

#[derive(Debug)]
pub enum Instruction {
    Eval(usize, Expression),
    GetField(usize, usize, Vec<FieldRef>), // dest, source, op
    AllocBuf(usize, usize),                // buf handle, len handle
    AllocDynBuf(usize),                    // buf handle
    WrapStream(Target, usize, Arc<Transform>, Vec<usize>), // stream, new stream, transformer, arguments
    ConditionalWrapStream(
        usize,
        Vec<Instruction>,
        Target,
        usize,
        usize,
        Arc<Transform>,
        Vec<usize>,
    ), // condition, prelude, stream, new stream, owned_new_stream, transformer, arguments
    EndStream(usize),

    EmitBuf(Target, usize),

    EncodeForeign(Target, usize, Arc<ForeignType>, Vec<usize>),
    EncodeRef(Target, usize, Vec<usize>),
    EncodeEnum(PrimitiveType, Target, usize),
    EncodeBitfield(Target, usize),
    EncodePrimitive(Target, usize, PrimitiveType),
    EncodePrimitiveArray(Target, usize, PrimitiveType, Option<usize>),
    // target, register of length
    Pad(Target, usize),

    // register representing iterator from -> term, term, inner
    Loop(usize, usize, Vec<Instruction>),
    // len target <- buffer, cast_type
    GetLen(usize, usize, Option<ScalarType>),
    Drop(usize),
    // original, checked, message
    NullCheck(usize, usize, String),
    Conditional(usize, Vec<Instruction>, Vec<Instruction>), // condition, if_true, if_false
    /// enum name, discriminant, original, checked, message
    UnwrapEnum(String, String, usize, usize, String),
    /// enum name, discriminant, original, checked: (enumstruct field name, checked), message
    UnwrapEnumStruct(String, String, usize, Vec<(String, usize)>, String),
    BreakBlock(Vec<Instruction>),
    Break,
}
