use super::*;

#[derive(Debug)]
pub enum Constructable {
    Struct {
        name: String,
        items: Vec<(String, usize)>,
    },
    Tuple(Vec<usize>),
    TaggedTuple {
        name: String,
        items: Vec<usize>,
    },
    TaggedEnum {
        name: String,
        discriminant: String,
        values: Vec<usize>,
    },
    TaggedEnumStruct {
        name: String,
        discriminant: String,
        values: Vec<(String, usize)>,
    },
}

#[derive(Debug)]
pub enum Instruction {
    Eval(usize, Expression, HashMap<String, usize>),
    Construct(usize, Constructable),
    // source, new_stream, len constraint
    Constrict(Target, usize, usize),
    WrapStream(Target, usize, Arc<Transform>, Vec<usize>), // stream, new stream, transformer, arguments
    ConditionalWrapStream(
        usize,
        Vec<Instruction>,
        Target,
        usize,
        Arc<Transform>,
        Vec<usize>,
    ), // condition, prelude, stream, new stream, transformer, arguments

    DecodeForeign(Target, usize, Arc<ForeignType>, Vec<usize>),
    DecodeRef(Target, usize, String, Vec<usize>),
    DecodeRepr(String, PrimitiveType, usize, Target),
    DecodePrimitive(Target, usize, PrimitiveType),
    DecodePrimitiveArray(Target, usize, PrimitiveType, Option<usize>),
    DecodeReprArray(Target, usize, String, PrimitiveType, Option<usize>),
    // target, register of length
    Skip(Target, usize),

    // register representing: internal stream, end index, terminator, output handle, inner
    Loop(
        Target,
        Option<usize>,
        Option<usize>,
        usize,
        Vec<Instruction>,
    ),
    LoopOutput(usize, usize), // output handle, item
    Conditional(Vec<usize>, Vec<usize>, usize, Vec<Instruction>), // target, interior_register, condition, if_true
    ConditionalPredicate(usize, Vec<Instruction>), // target, interior_register, condition, if_true
    /// returns from decoder early
    Return(usize),
    Error(String),
}
