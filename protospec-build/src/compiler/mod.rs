use crate::asg::*;
use crate::coder;
use crate::{BinaryOp, UnaryOp};
use expr::*;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use quote::{format_ident, quote};
use std::{sync::Arc, unimplemented};

mod decoder;
mod encoder;
mod expr;

pub fn global_name(input: &str) -> String {
    input.to_string()
}

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub enum_derives: Vec<String>,
    pub struct_derives: Vec<String>,
    pub include_async: bool,
    pub use_anyhow: bool,
    pub debug_mode: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            include_async: false,
            debug_mode: false,
            enum_derives: vec![
                "PartialEq".to_string(),
                "Debug".to_string(),
                "Clone".to_string(),
                "Default".to_string(),
            ],
            struct_derives: vec![
                "PartialEq".to_string(),
                "Debug".to_string(),
                "Clone".to_string(),
                "Default".to_string(),
            ],
            use_anyhow: false,
        }
    }
}

impl CompileOptions {
    fn emit_struct_derives(&self, extra: &[&str]) -> TokenStream {
        let mut all: Vec<_> = self.struct_derives.iter().map(|x| &**x).collect();
        all.extend_from_slice(extra);
        all.sort();
        all.dedup();

        self.emit_derives(&all[..])
    }

    fn emit_enum_derives(&self, extra: &[&str]) -> TokenStream {
        let mut all: Vec<_> = self.enum_derives.iter().map(|x| &**x).collect();
        all.extend_from_slice(extra);
        all.retain(|x| *x != "Default");
        all.sort();
        all.dedup();

        self.emit_derives(&all[..])
    }

    fn emit_derives(&self, all: &[&str]) -> TokenStream {
        if all.len() > 0 {
            let items = flatten(
                all.into_iter()
                    .map(|x| {
                        let ident = emit_ident(x);
                        quote! {
                            #ident,
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            quote! {
                #[derive(#items)]
            }
        } else {
            quote! {}
        }
    }
}

pub fn compile_program(program: &Program, options: &CompileOptions) -> TokenStream {
    let mut components = vec![];
    let errors = if options.use_anyhow {
        quote! {
            pub type Result<T> = anyhow::Result<T>;
    
            fn encode_error<S: AsRef<str>>(value: S) -> anyhow::Error {
                anyhow::anyhow!("{}", value.as_ref())
            }

            fn decode_error<S: AsRef<str>>(value: S) -> anyhow::Error {
                anyhow::anyhow!("{}", value.as_ref())
            }
        }
    } else {
        quote! {
            use std::error::Error;
            pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
    
            #[derive(Debug)]
            pub struct DecodeError(pub String);
            impl std::fmt::Display for DecodeError {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
            impl Error for DecodeError {}
            #[derive(Debug)]
            pub struct EncodeError(pub String);
            impl std::fmt::Display for EncodeError {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
            impl Error for EncodeError {}    

            fn encode_error<S: AsRef<str>>(value: S) -> EncodeError {
                EncodeError(value.as_ref().to_string())
            }

            fn decode_error<S: AsRef<str>>(value: S) -> DecodeError {
                DecodeError(value.as_ref().to_string())
            }
        }
    };

    components.push(quote! {
        use std::io::{Read, BufRead, Cursor};
        use std::slice;
        use std::mem;
        use std::convert::TryInto;

        #errors
    });
    for (name, field) in program.types.iter() {
        match &*field.type_.borrow() {
            Type::Foreign(_) => continue,
            Type::Container(item) => {
                components.push(generate_container(&name, &**item, options));
            }
            Type::Enum(item) => {
                components.push(generate_enum(&name, item, options));
            }
            Type::Bitfield(item) => {
                components.push(generate_bitfield(&name, item, options));
            }
            generic => {
                let ident = format_ident!("{}", global_name(name));
                let type_ref = emit_type_ref(generic);
                let type_ref = if field.condition.borrow().is_some() {
                    quote! {
                        Option<#type_ref>
                    }
                } else {
                    type_ref
                };
                let derives = options.emit_struct_derives(&[]);

                components.push(quote! {
                    #derives
                    pub struct #ident(pub #type_ref);
                });
            }
        }
        components.push(prepare_impls(&field, options));
    }
    let components = flatten(components);
    quote! {
        #[allow(unused_imports, unused_parens, unused_variables, dead_code, unused_mut)]
        mod _ps {
            #components
        }
        pub use _ps::*;
    }
}

fn ref_resolver(_f: &Arc<Field>) -> TokenStream {
    unimplemented!("cannot reference field in input default");
}

fn prepare_impls(field: &Arc<Field>, options: &CompileOptions) -> TokenStream {
    let container_ident = format_ident!("{}", global_name(&field.name));

    let mut decode_context = coder::decode::Context::new();
    decode_context.decode_field_top(field);
    let decode_sync = decoder::prepare_decoder(options, &decode_context, false);

    let mut new_context = coder::encode::Context::new();
    new_context.encode_field_top(field);

    let encode_sync = encoder::prepare_encoder(&new_context, false);

    let mut arguments = vec![];
    let mut redefaults = vec![];
    for argument in field.arguments.borrow().iter() {
        let name = emit_ident(&argument.name);
        let type_ref = emit_type_ref(&argument.type_);
        let opt_type_ref = if argument.default_value.is_some() {
            quote! { Option<#type_ref> }
        } else {
            type_ref.clone()
        };
        arguments.push(quote! {, #name: #opt_type_ref});
        if let Some(default_value) = argument.default_value.as_ref() {
            let emitted = emit_expression(default_value, &ref_resolver);
            redefaults.push(quote! {
                let #name: #type_ref = if let Some(#name) = #name {
                    #name
                } else {
                    #emitted
                };
            })
        }
    }
    let arguments = flatten(arguments);
    let redefaults = flatten(redefaults);

    let async_functions = if options.include_async {
        let async_recursion = if field.is_maybe_cyclical.get() {
            quote! {
                #[async_recursion::async_recursion]
            }
        } else {
            quote! {}
        };

        let encode_async = encoder::prepare_encoder(&new_context, true);
        let decode_async = decoder::prepare_decoder(options, &decode_context, true);
        quote! {
            #async_recursion
            pub async fn encode_async<W: tokio::io::AsyncWrite + Send + Sync + Unpin>(&self, writer: &mut W #arguments) -> Result<()> {
                #redefaults
                #encode_async
            }

            #async_recursion
            pub async fn decode_async<R: tokio::io::AsyncBufRead + Send + Sync + Unpin>(reader: &mut R #arguments) -> Result<Self> {
                #redefaults
                #decode_async
            }
        }
    } else {
        quote! {}
    };

    quote! {
        impl #container_ident {
            pub fn decode_sync<R: Read + BufRead>(reader: &mut R #arguments) -> Result<Self> {
                #redefaults
                #decode_sync
            }

            pub fn encode_sync<W: std::io::Write>(&self, writer: &mut W #arguments) -> Result<()> {
                #redefaults
                #encode_sync
            }

            #async_functions
        }
    }
}

fn emit_ident(name: &str) -> TokenStream {
    let ident = format_ident!("{}", name);
    quote! {
        #ident
    }
}

fn emit_register(register: usize) -> TokenStream {
    let ident = format_ident!("r_{}", register);
    quote! {
        #ident
    }
}

fn flatten<T: IntoIterator<Item = TokenStream>>(iter: T) -> TokenStream {
    let mut out = quote! {};
    out.append_all(iter);
    out
}

pub fn emit_type_ref(item: &Type) -> TokenStream {
    match item {
        Type::Container(_) => unimplemented!(),
        Type::Enum(_) => unimplemented!(),
        Type::Bitfield(_) => unimplemented!(),
        Type::Scalar(s) => emit_ident(&s.to_string()),
        Type::Array(array_type) => {
            let interior = emit_type_ref(&array_type.element.type_.borrow());
            quote! {
                Vec<#interior>
            }
        }
        Type::Foreign(f) => f.obj.type_ref(),
        Type::F32 => emit_ident("f32"),
        Type::F64 => emit_ident("f64"),
        Type::Bool => emit_ident("bool"),
        Type::Ref(field) => match &*field.target.type_.borrow() {
            Type::Foreign(c) => c.obj.type_ref(),
            _ => emit_ident(&global_name(&field.target.name)),
        },
    }
}

fn generate_container_fields(access: TokenStream, item: &ContainerType) -> TokenStream {
    let mut fields = vec![];
    for (name, field) in item.flatten_view() {
        if field.is_pad.get() {
            continue;
        }
        let name_ident = format_ident!("{}", name);
        let type_ref = emit_type_ref(&field.type_.borrow());
        let type_ref = if field.condition.borrow().is_some() {
            quote! {
                Option<#type_ref>
            }
        } else {
            type_ref
        };

        fields.push(quote! {
            #access #name_ident: #type_ref,
        });
    }
    flatten(fields)
}

pub fn generate_container(
    name: &str,
    item: &ContainerType,
    options: &CompileOptions,
) -> TokenStream {
    let name_ident = format_ident!("{}", global_name(name));
    if item.is_enum.get() {
        let derives = options.emit_enum_derives(&[]);
        let mut fields = vec![];
        for (name, field) in &item.items {
            let name_ident = format_ident!("{}", name);
            let type_ = field.type_.borrow();
            let type_ref = match &*type_ {
                Type::Container(sub_container) => {
                    let subfields = generate_container_fields(quote! { }, &**sub_container);
                    quote! {
                        {
                            #subfields
                        }
                    }
                },
                type_ => {
                    let emitted = emit_type_ref(type_);
                    quote! { (#emitted) }
                }
            };
    
            fields.push(quote! {
                #name_ident#type_ref,
            });
        }
        let fields = flatten(fields);

        let default_impl = if options.enum_derives.iter().any(|x| x == "Default") {
            let (default_field, field) = item.items.first().expect("missing enum entry for default");
            let default_field = format_ident!("{}", default_field);

            let type_ = field.type_.borrow();
            let default_value = match &*type_ {
                Type::Container(sub_container) => {
                    let mut fields = vec![];
                    for (name, _) in sub_container.flatten_view() {
                        let name_ident = format_ident!("{}", name);
                
                        fields.push(quote! {
                            #name_ident: Default::default(),
                        });
                    }
                    let fields = flatten(fields);
                    quote! {
                        {
                            #fields
                        }
                    }
                },
                _ => {
                    quote! { (Default::default()) }
                }
            };

            quote! {
                impl Default for #name_ident {
                    fn default() -> Self {
                        Self::#default_field#default_value
                    }
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #derives
            pub enum #name_ident {
                #fields
            }

            #default_impl
        }
    } else {
        let derives = options.emit_struct_derives(&[]);
        let fields = generate_container_fields(quote! { pub }, item);
    
        quote! {
            #derives
            pub struct #name_ident {
                #fields
            }
        }
    }
}

pub fn generate_enum(name: &str, item: &EnumType, options: &CompileOptions) -> TokenStream {
    let name_ident = format_ident!("{}", global_name(name));
    let mut fields = vec![];
    let mut from_repr_matches = vec![];
    for (name, cons) in item.items.iter() {
        let value_ident = format_ident!("{}", name);
        let value = eval_const_expression(&cons.value);
        if value.is_none() {
            unimplemented!("could not resolve constant expression");
        }
        let value = value.unwrap();
        let value = value.emit();
        fields.push(quote! {
            #value_ident = #value,
        });
        from_repr_matches.push(quote! {
            #value => Ok(#name_ident::#value_ident),
        })
    }
    let fields = flatten(fields);

    let from_repr_matches = flatten(from_repr_matches);
    let rep = format_ident!("{}", item.rep.to_string());
    let rep_size = item.rep.size() as usize;
    let derives = options.emit_enum_derives(&["Clone", "Copy"]);

    let format_string = format!("illegal enum value '{{}}' for enum '{}'", name);

    let default_impl = if options.enum_derives.iter().any(|x| x == "Default") {
        let (default_field, _) = item.items.first().expect("missing enum entry for default");
        let default_field = format_ident!("{}", default_field);
        quote! {
            impl Default for #name_ident {
                fn default() -> Self {
                    Self::#default_field
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[repr(#rep)]
        #derives
        pub enum #name_ident {
            #fields
        }

        impl #name_ident {
            pub fn from_repr(repr: #rep) -> Result<Self> {
                match repr {
                    #from_repr_matches
                    x => Err(decode_error(format!(#format_string, x)).into()),
                }
            }

            pub fn to_be_bytes(&self) -> [u8; #rep_size] {
                (self as #rep).to_be_bytes()
            }
        }

        #default_impl
    }
}

pub fn generate_bitfield(name: &str, item: &BitfieldType, options: &CompileOptions) -> TokenStream {
    let name_ident = format_ident!("{}", global_name(name));
    let mut fields = vec![];
    let mut all_fields = ConstInt::parse(item.rep, "0", crate::Span::default()).unwrap();
    let zero = all_fields;

    for (name, cons) in item.items.iter() {
        let value_ident = format_ident!("{}", name);
        let value = eval_const_expression(&cons.value);
        if value.is_none() {
            unimplemented!("could not resolve constant expression");
        }
        let value = value.unwrap();
        let int_value = match &value {
            ConstValue::Int(x) => *x,
            _ => panic!("invalid const value type"),
        };
        if (int_value & all_fields).unwrap() != zero {
            panic!("overlapping bit fields");
        }
        all_fields = (all_fields | int_value).unwrap();

        let value = value.emit();
        fields.push(quote! {
            pub const #value_ident: Self = Self(#value);
        });
    }
    let fields = flatten(fields);

    let rep = format_ident!("{}", item.rep.to_string());
    let rep_size = item.rep.size() as usize;
    let derives = options.emit_struct_derives(&["Clone", "Copy", "Default"]);

    let format_string = format!("illegal bitfield value '{{}}' for bitfield '{}'", name);
    let all_fields = ConstValue::Int(all_fields).emit();

    quote! {
        #[repr(transparent)]
        #derives
        pub struct #name_ident(pub #rep);

        impl #name_ident {
            #fields
            pub const ALL: Self = Self(#all_fields);

            pub fn from_repr(repr: #rep) -> Result<Self> {
                if (repr & !Self::ALL.0) != 0 {
                    Err(decode_error(format!(#format_string, repr)).into())
                } else {
                    Ok(Self(repr))
                }
            }

            pub fn to_be_bytes(&self) -> [u8; #rep_size] {
                self.0.to_be_bytes()
            }
        }

        impl core::ops::BitOr for #name_ident {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitAnd for #name_ident {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitXor for #name_ident {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }

        impl core::ops::Not for #name_ident {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }
    }
}
