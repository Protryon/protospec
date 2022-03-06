use std::fmt;
use std::fmt::Write;

use indenter::indented;

use super::*;

#[derive(Debug)]
pub enum Instruction {
    /// dest, expression
    Eval(usize, Expression),
    /// dest, source, ops
    GetField(usize, usize, Vec<FieldRef>),
    /// dest, ref name
    GetRef(usize, String),
    /// ref name, value
    SetRef(String, usize),
    /// buf handle, len register
    AllocBuf(usize, usize),
    /// buf handle
    AllocDynBuf(usize),
    /// stream, new stream, transformer, arguments
    WrapStream(Target, usize, Arc<Transform>, Vec<usize>),
    /// condition, prelude, stream, new stream, owned_new_stream, transformer, arguments
    ConditionalWrapStream(
        usize,
        Vec<Instruction>,
        Target,
        usize,
        usize,
        Arc<Transform>,
        Vec<usize>,
    ),
    /// stream
    EndStream(usize),

    /// dest, buf handle
    EmitBuf(Target, usize),

    /// dest, source, type, arguments
    EncodeForeign(Target, usize, Arc<ForeignType>, Vec<usize>),
    /// dest, source, arguments
    EncodeRef(Target, usize, Vec<usize>),
    /// rep type, dest, source
    EncodeEnum(Target, usize, EndianScalarType),
    /// dest, source
    EncodeBitfield(Target, usize, EndianScalarType),
    /// dest, source, type
    EncodePrimitive(Target, usize, PrimitiveType),
    /// dest, source, element type, known length if any
    EncodePrimitiveArray(Target, usize, PrimitiveType, Option<usize>),
    /// dest, source, element type, known length if any
    EncodeReprArray(Target, usize, PrimitiveType, Option<usize>),
    /// dest, length register
    Pad(Target, usize),

    /// register representing iterator from -> term, term, inner
    Loop(usize, usize, Vec<Instruction>),
    /// len target, buffer, cast_type
    GetLen(usize, usize, Option<ScalarType>),
    /// register
    Drop(usize),
    /// original, checked, is_copyable, message
    NullCheck(usize, usize, bool, String),
    /// condition, if_true, if_false
    Conditional(usize, Vec<Instruction>, Vec<Instruction>),
    /// enum name, discriminant, original, checked, message
    UnwrapEnum(String, String, usize, usize, String),
    /// enum name, discriminant, original, checked: (enumstruct field name, checked, do_copy), message
    UnwrapEnumStruct(String, String, usize, Vec<(String, usize, bool)>, String),
    /// instructions
    BreakBlock(Vec<Instruction>),
    Break,
}

fn write_arguments(f: &mut fmt::Formatter<'_>, arguments: &[usize]) -> fmt::Result {
    if let Some(first) = arguments.first() {
        write!(f, "{}", first)?;
    }
    for argument in arguments.iter().skip(1) {
        write!(f, ",{}", argument)?;
    }
    Ok(())
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Eval(dest, expression) => write!(f, "Eval({}, {:?})", dest, expression),
            Instruction::GetField(dest, source, ops) => {
                write!(f, "GetField({}, {}, {:?})", dest, source, ops)
            }
            Instruction::GetRef(dest, ref_name) => write!(f, "GetRef({}, '{}')", dest, ref_name),
            Instruction::SetRef(ref_name, source) => {
                write!(f, "SetRef('{}', {})", ref_name, source)
            }
            Instruction::AllocBuf(buf, len) => write!(f, "AllocBuf({}, {})", buf, len),
            Instruction::AllocDynBuf(buf) => write!(f, "AllocDynBuf({})", buf),
            Instruction::WrapStream(stream, new_stream, transform, arguments) => {
                write!(
                    f,
                    "WrapStream({:?}, {}, '{}', ",
                    stream, new_stream, transform.name
                )?;
                write_arguments(f, &arguments[..])?;
                write!(f, ")")
            }
            Instruction::ConditionalWrapStream(
                condition,
                prelude,
                stream,
                new_stream,
                owned_new_stream,
                transform,
                arguments,
            ) => {
                write!(
                    f,
                    "ConditionalWrapStream({}, {:?}, {}, {}, '{}', ",
                    condition, stream, new_stream, owned_new_stream, transform.name
                )?;
                write_arguments(f, &arguments[..])?;
                write!(f, ")")?;
                for instruction in prelude {
                    write!(indented(f), "\n{}", instruction)?;
                }
                Ok(())
            }
            Instruction::EndStream(stream) => write!(f, "EndStream({})", stream),
            Instruction::EmitBuf(dest, buf_handle) => {
                write!(f, "EmitBuf({:?}, {})", dest, buf_handle)
            }
            Instruction::EncodeForeign(dest, source, type_, arguments) => {
                write!(
                    f,
                    "EncodeForeign({:?}, {}, '{}', ",
                    dest, source, type_.name
                )?;
                write_arguments(f, &arguments[..])?;
                write!(f, ")")
            }
            Instruction::EncodeRef(dest, source, arguments) => {
                write!(f, "EncodeRef({:?}, {}, ", dest, source)?;
                write_arguments(f, &arguments[..])?;
                write!(f, ")")
            }
            Instruction::EncodeEnum(dest, source, type_) => {
                write!(f, "EncodeEnum({:?}, {}, {})", dest, source, type_)
            }
            Instruction::EncodeBitfield(dest, source, type_) => {
                write!(f, "EncodeBitfield({:?}, {}, {})", dest, source, type_)
            }
            Instruction::EncodePrimitive(dest, source, type_) => {
                write!(f, "EncodePrimitive({:?}, {}, {})", dest, source, type_)
            }
            Instruction::EncodePrimitiveArray(dest, source, element_type, length) => write!(
                f,
                "EncodePrimitiveArray({:?}, {}, {}, {:?})",
                dest, source, element_type, length
            ),
            Instruction::EncodeReprArray(dest, source, element_type, length) => write!(
                f,
                "EncodeReprArray({:?}, {}, {}, {:?})",
                dest, source, element_type, length
            ),
            Instruction::Pad(dest, length) => write!(f, "Pad({:?}, {})", dest, length),
            Instruction::Loop(inner, end, instructions) => {
                write!(f, "Loop({}, {})", inner, end)?;
                for instruction in instructions {
                    write!(indented(f), "\n{}", instruction)?;
                }
                Ok(())
            }
            Instruction::GetLen(dest, buffer, cast_type) => {
                write!(f, "GetLen({}, {}, {:?})", dest, buffer, cast_type)
            }
            Instruction::Drop(register) => write!(f, "Drop({})", register),
            Instruction::NullCheck(original, checked, is_copyable, message) => write!(
                f,
                "NullCheck({}, {}, {}, {})",
                original, checked, is_copyable, message
            ),
            Instruction::Conditional(condition, if_true, if_false) => {
                write!(f, "Conditional({})", condition)?;
                for instruction in if_true {
                    write!(indented(f), "\n{}", instruction)?;
                }
                write!(f, "\nElse")?;
                for instruction in if_false {
                    write!(indented(f), "\n{}", instruction)?;
                }
                Ok(())
            }
            Instruction::UnwrapEnum(enum_name, discriminant, original, checked, message) => write!(
                f,
                "UnwrapEnum('{}', '{}', {}, {}, '{}')",
                enum_name, discriminant, original, checked, message
            ),
            Instruction::UnwrapEnumStruct(enum_name, discriminant, original, checked, message) => {
                write!(
                    f,
                    "UnwrapEnum('{}', '{}', {}, '{}')",
                    enum_name, discriminant, original, message
                )?;
                for (name, register, do_copy) in checked {
                    write!(
                        indented(f),
                        "\n{}: {}{}",
                        name,
                        if *do_copy { "*" } else { "" },
                        register
                    )?;
                }
                Ok(())
            }
            Instruction::BreakBlock(instructions) => {
                write!(f, "BreakBlock()")?;
                for instruction in instructions {
                    write!(indented(f), "\n{}", instruction)?;
                }
                Ok(())
            }
            Instruction::Break => write!(f, "Break()"),
        }
    }
}
