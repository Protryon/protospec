use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstInt {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

macro_rules! const_int_biop {
    ($e1: expr, $e2: expr, $i1: ident, $i2: ident, $op: expr) => {
        match ($e1, $e2) {
            (ConstInt::I8($i1), ConstInt::I8($i2)) => $op,
            (ConstInt::I16($i1), ConstInt::I16($i2)) => $op,
            (ConstInt::I32($i1), ConstInt::I32($i2)) => $op,
            (ConstInt::I64($i1), ConstInt::I64($i2)) => $op,
            (ConstInt::I128($i1), ConstInt::I128($i2)) => $op,
            (ConstInt::U8($i1), ConstInt::U8($i2)) => $op,
            (ConstInt::U16($i1), ConstInt::U16($i2)) => $op,
            (ConstInt::U32($i1), ConstInt::U32($i2)) => $op,
            (ConstInt::U64($i1), ConstInt::U64($i2)) => $op,
            (ConstInt::U128($i1), ConstInt::U128($i2)) => $op,
            _ => None,
        }
    };
}

macro_rules! const_int_biop_map {
    ($e1: expr, $e2: expr, $i1: ident, $i2: ident, $op: expr) => {
        match ($e1, $e2) {
            (ConstInt::I8($i1), ConstInt::I8($i2)) => Some(ConstInt::I8($op)),
            (ConstInt::I16($i1), ConstInt::I16($i2)) => Some(ConstInt::I16($op)),
            (ConstInt::I32($i1), ConstInt::I32($i2)) => Some(ConstInt::I32($op)),
            (ConstInt::I64($i1), ConstInt::I64($i2)) => Some(ConstInt::I64($op)),
            (ConstInt::I128($i1), ConstInt::I128($i2)) => Some(ConstInt::I128($op)),
            (ConstInt::U8($i1), ConstInt::U8($i2)) => Some(ConstInt::U8($op)),
            (ConstInt::U16($i1), ConstInt::U16($i2)) => Some(ConstInt::U16($op)),
            (ConstInt::U32($i1), ConstInt::U32($i2)) => Some(ConstInt::U32($op)),
            (ConstInt::U64($i1), ConstInt::U64($i2)) => Some(ConstInt::U64($op)),
            (ConstInt::U128($i1), ConstInt::U128($i2)) => Some(ConstInt::U128($op)),
            _ => None,
        }
    };
}

macro_rules! const_int_map {
    ($e1: expr, $i1: ident, $sop: expr, $usop: expr) => {
        match $e1 {
            ConstInt::I8($i1) => Some(ConstInt::I8($sop)),
            ConstInt::I16($i1) => Some(ConstInt::I16($sop)),
            ConstInt::I32($i1) => Some(ConstInt::I32($sop)),
            ConstInt::I64($i1) => Some(ConstInt::I64($sop)),
            ConstInt::I128($i1) => Some(ConstInt::I128($sop)),
            ConstInt::U8($i1) => Some(ConstInt::U8($usop)),
            ConstInt::U16($i1) => Some(ConstInt::U16($usop)),
            ConstInt::U32($i1) => Some(ConstInt::U32($usop)),
            ConstInt::U64($i1) => Some(ConstInt::U64($usop)),
            ConstInt::U128($i1) => Some(ConstInt::U128($usop)),
        }
    };
}

macro_rules! const_int_op {
    ($e1: expr, $i1: ident, $op: expr) => {
        match $e1 {
            ConstInt::I8($i1) => $op,
            ConstInt::I16($i1) => $op,
            ConstInt::I32($i1) => $op,
            ConstInt::I64($i1) => $op,
            ConstInt::I128($i1) => $op,
            ConstInt::U8($i1) => $op,
            ConstInt::U16($i1) => $op,
            ConstInt::U32($i1) => $op,
            ConstInt::U64($i1) => $op,
            ConstInt::U128($i1) => $op,
        }
    };
}

impl PartialOrd for ConstInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        const_int_biop!(self, other, i1, i2, i1.partial_cmp(i2))
    }
}

impl BitOr for ConstInt {
    type Output = Option<Self>;

    fn bitor(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 | i2)
    }
}

impl BitAnd for ConstInt {
    type Output = Option<Self>;

    fn bitand(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 & i2)
    }
}

impl BitXor for ConstInt {
    type Output = Option<Self>;

    fn bitxor(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 ^ i2)
    }
}

impl Shr for ConstInt {
    type Output = Option<Self>;

    fn shr(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 >> i2)
    }
}

impl Shl for ConstInt {
    type Output = Option<Self>;

    fn shl(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 << i2)
    }
}

impl Add for ConstInt {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 + i2)
    }
}

impl Sub for ConstInt {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 - i2)
    }
}

impl Mul for ConstInt {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 * i2)
    }
}

impl Div for ConstInt {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 / i2)
    }
}

impl Rem for ConstInt {
    type Output = Option<Self>;

    fn rem(self, other: Self) -> Self::Output {
        const_int_biop_map!(self, other, i1, i2, i1 % i2)
    }
}

impl Neg for ConstInt {
    type Output = Option<Self>;

    #[allow(unreachable_code, unused_variables)]
    fn neg(self) -> Self::Output {
        const_int_map!(self, i1, -i1, unimplemented!("cannot neg unsigned value"))
    }
}

impl Not for ConstInt {
    type Output = Option<Self>;

    fn not(self) -> Self::Output {
        const_int_map!(self, i1, i1.not(), i1.not())
    }
}

impl ConstInt {
    pub fn cast_to(&self, target: ScalarType) -> Self {
        const_int_op!(
            self,
            i1,
            match target {
                ScalarType::I8 => ConstInt::I8(*i1 as i8),
                ScalarType::I16 => ConstInt::I16(*i1 as i16),
                ScalarType::I32 => ConstInt::I32(*i1 as i32),
                ScalarType::I64 => ConstInt::I64(*i1 as i64),
                ScalarType::I128 => ConstInt::I128(*i1 as i128),
                ScalarType::U8 => ConstInt::U8(*i1 as u8),
                ScalarType::U16 => ConstInt::U16(*i1 as u16),
                ScalarType::U32 => ConstInt::U32(*i1 as u32),
                ScalarType::U64 => ConstInt::U64(*i1 as u64),
                ScalarType::U128 => ConstInt::U128(*i1 as u128),
            }
        )
    }

    pub fn parse(scalar_type: ScalarType, value: &str, span: Span) -> AsgResult<ConstInt> {
        if value.starts_with("0x") {
            let value = &value[2..];
            return Ok(match scalar_type {
                ScalarType::I8 => ConstInt::I8(
                    i8::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I16 => ConstInt::I16(
                    i16::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I32 => ConstInt::I32(
                    i32::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I64 => ConstInt::I64(
                    i64::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::I128 => ConstInt::I128(
                    i128::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U8 => ConstInt::U8(
                    u8::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U16 => ConstInt::U16(
                    u16::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U32 => ConstInt::U32(
                    u32::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U64 => ConstInt::U64(
                    u64::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::U128 => ConstInt::U128(
                    u128::from_str_radix(value, 16)
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
            });
        }
        Ok(match scalar_type {
            ScalarType::I8 => ConstInt::I8(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I16 => ConstInt::I16(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I32 => ConstInt::I32(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I64 => ConstInt::I64(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::I128 => ConstInt::I128(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U8 => ConstInt::U8(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U16 => ConstInt::U16(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U32 => ConstInt::U32(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U64 => ConstInt::U64(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::U128 => ConstInt::U128(
                value
                    .parse()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
        })
    }
}
