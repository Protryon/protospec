use super::*;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum ConstFloat {
    F32(f32),
    F64(f64),
}

macro_rules! const_float_op {
    ($e1: expr, $i1: ident, $op: expr) => {
        match $e1 {
            ConstFloat::F32($i1) => $op,
            ConstFloat::F64($i1) => $op,
        }
    };
}

macro_rules! const_float_map {
    ($e1: expr, $i1: ident, $sop: expr, $usop: expr) => {
        match $e1 {
            ConstFloat::F32($i1) => Some(ConstFloat::F32($usop)),
            ConstFloat::F64($i1) => Some(ConstFloat::F64($usop)),
        }
    };
}

macro_rules! const_float_biop_map {
    ($e1: expr, $e2: expr, $i1: ident, $i2: ident, $op: expr) => {
        match ($e1, $e2) {
            (ConstFloat::F32($i1), ConstFloat::F32($i2)) => Some(ConstFloat::F32($op)),
            (ConstFloat::F64($i1), ConstFloat::F64($i2)) => Some(ConstFloat::F64($op)),
            _ => None,
        }
    };
}

impl Add for ConstFloat {
    type Output = Option<Self>;

    fn add(self, other: Self) -> Self::Output {
        const_float_biop_map!(self, other, i1, i2, i1 + i2)
    }
}

impl Sub for ConstFloat {
    type Output = Option<Self>;

    fn sub(self, other: Self) -> Self::Output {
        const_float_biop_map!(self, other, i1, i2, i1 - i2)
    }
}

impl Mul for ConstFloat {
    type Output = Option<Self>;

    fn mul(self, other: Self) -> Self::Output {
        const_float_biop_map!(self, other, i1, i2, i1 * i2)
    }
}

impl Div for ConstFloat {
    type Output = Option<Self>;

    fn div(self, other: Self) -> Self::Output {
        const_float_biop_map!(self, other, i1, i2, i1 / i2)
    }
}

impl Rem for ConstFloat {
    type Output = Option<Self>;

    fn rem(self, other: Self) -> Self::Output {
        const_float_biop_map!(self, other, i1, i2, i1 % i2)
    }
}

impl Neg for ConstFloat {
    type Output = Option<Self>;

    #[allow(unreachable_code, unused_variables)]
    fn neg(self) -> Self::Output {
        const_float_map!(self, i1, -i1, unimplemented!("cannot neg unsigned value"))
    }
}


impl ConstFloat {
    pub fn cast_to(&self, target: ScalarType) -> Self {
        const_float_op!(
            self,
            i1,
            match target {
                ScalarType::F32 => ConstFloat::F32(*i1 as f32),
                ScalarType::F64 => ConstFloat::F64(*i1 as f64),
                _ => panic!("This isn't a float, but an int!"),
            }
        )
    }

    pub fn parse(scalar_type: ScalarType, value: &str, span: Span) -> AsgResult<ConstFloat> {
        if value.starts_with("0x") {
            let value = &value[2..];
            return Ok(match scalar_type {
                ScalarType::F32 => ConstFloat::F32(
                    value
                        .parse::<f32>()
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                ScalarType::F64 => ConstFloat::F64(
                    value
                        .parse::<f64>()
                        .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
                ),
                _ => {
                    panic!("This isn't a float, but an int!")
                }
            });
        }
        Ok(match scalar_type {
            ScalarType::F32 => ConstFloat::F32(
                value
                    .parse::<f32>()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            ScalarType::F64 => ConstFloat::F64(
                value
                    .parse::<f64>()
                    .map_err(|_| AsgError::InvalidInt(value.to_string(), span))?,
            ),
            _ => {
                panic!("This isn't a float, but an int!")
            }
        })
    }
}
