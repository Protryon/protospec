use super::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Copy, Debug)]
pub enum ScalarType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Copy, Debug)]
pub struct EndianScalarType {
    pub scalar: ScalarType,
    pub endian: Endian,
}

impl EndianScalarType {
    pub fn is_little_endian(&self) -> bool {
        matches!(self.endian, Endian::Little)
    }
}

impl ScalarType {
    pub fn can_implicit_cast_to(&self, to: &ScalarType) -> bool {
        if self.is_signed() != to.is_signed() {
            return false;
        }
        if self.size() > to.size() {
            return false;
        }
        true
    }

    pub fn is_signed(&self) -> bool {
        match self {
            ScalarType::I8 => true,
            ScalarType::I16 => true,
            ScalarType::I32 => true,
            ScalarType::I64 => true,
            ScalarType::I128 => true,
            _ => false,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            ScalarType::I8 | ScalarType::U8 => 1,
            ScalarType::I16 | ScalarType::U16 => 2,
            ScalarType::I32 | ScalarType::U32 => 4,
            ScalarType::I64 | ScalarType::U64 => 8,
            ScalarType::I128 | ScalarType::U128 => 16,
            ScalarType::F32 => 4,
            ScalarType::F64 => 8,
        }
    }
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScalarType::*;
        write!(
            f,
            "{}",
            match self {
                U8 => "u8",
                U16 => "u16",
                U32 => "u32",
                U64 => "u64",
                U128 => "u128",
                I8 => "i8",
                I16 => "i16",
                I32 => "i32",
                I64 => "i64",
                I128 => "i128",
                F32 => "f32",
                F64 => "f64",
            }
        )
    }
}

impl fmt::Display for Endian {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Endian::*;
        write!(
            f,
            "{}",
            match self {
                Big => "",
                Little => "le",
            }
        )
    }
}

impl fmt::Display for EndianScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.scalar, self.endian)
    }
}

impl Into<EndianScalarType> for ScalarType {
    fn into(self) -> EndianScalarType {
        EndianScalarType {
            scalar: self,
            endian: Endian::Big,
        }
    }
}

impl Into<ScalarType> for EndianScalarType {
    fn into(self) -> ScalarType {
        self.scalar
    }
}
