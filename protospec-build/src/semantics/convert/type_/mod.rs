use super::*;

mod container;

mod enum_;

mod bitfield;

mod array;

mod type_ref;

#[derive(Clone, Debug)]
pub enum TypePurpose {
    TypeDefinition(String),
    ConstDefinition,
    ArrayInterior,
    FieldInterior,
    Expression,
}

impl Scope {
    pub fn convert_ast_type(
        self_: &Arc<RefCell<Scope>>,
        typ: &ast::RawType,
        purpose: TypePurpose,
    ) -> AsgResult<Type> {
        Ok(match typ {
            ast::RawType::Container(type_) => {
                if matches!(purpose, TypePurpose::ArrayInterior) {
                    return Err(AsgError::InlineRepetition(type_.span));
                }
                Self::convert_container_type(self_, type_, purpose)?
            }
            ast::RawType::Enum(type_) => {
                if !matches!(purpose, TypePurpose::TypeDefinition(_)) {
                    return Err(AsgError::MustBeToplevel(type_.span));
                }
                Self::convert_enum_type(self_, type_, purpose)?
            }
            ast::RawType::Bitfield(type_) => {
                if !matches!(purpose, TypePurpose::TypeDefinition(_)) {
                    return Err(AsgError::MustBeToplevel(type_.span));
                }
                Self::convert_bitfield_type(self_, type_, purpose)?
            }
            ast::RawType::Scalar(type_) => Type::Scalar(type_.clone()),
            ast::RawType::Array(type_) => Self::convert_array_type(self_, type_)?,
            ast::RawType::F32 => Type::F32,
            ast::RawType::F64 => Type::F64,
            ast::RawType::Bool => Type::Bool,
            ast::RawType::Ref(type_) => Self::convert_type_ref_type(self_, type_)?,
        })
    }
}
