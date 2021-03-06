use crate::{asg::*};
use crate::coder;
use crate::{BinaryOp, UnaryOp};
use expr::*;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use quote::{format_ident, quote};
use std::{sync::Arc, unimplemented};

mod decoder_sync;
mod encoder_sync;
mod expr;

pub fn global_name(input: &str) -> String {
    input.to_string()
}

#[derive(Default, Clone, Debug)]
pub struct CompileOptions {
    pub derives: Vec<String>,
}

impl CompileOptions {
    fn emit_derives(&self, extra: &[&str]) -> TokenStream {
        let mut all: Vec<_> = self.derives.iter().map(|x| &**x).collect();
        all.extend_from_slice(extra);
        all.sort();
        all.dedup();

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
    //let mut emitted_types = IndexMap<Uuid,
    let mut components = vec![];
    components.push(quote! {
        use std::io::{Read, Write, BufRead, Cursor};
        use std::slice;
        use std::mem;
        use std::convert::TryInto;
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
                let derives = options.emit_derives(&[]);

                components.push(quote! {
                    #derives
                    pub struct #ident(pub #type_ref);
                });
            }
        }
        components.push(prepare_impls(&field));
    }
    let components = flatten(components);
    quote! {
        #components
    }
}

fn ref_resolver(f: &Arc<Field>) -> TokenStream {
    unimplemented!("cannot reference field in input default");
}

fn prepare_impls(field: &Arc<Field>) -> TokenStream {
    let container_ident = format_ident!("{}", global_name(&field.name));
    // let mut context = OldContext::new();
    // context.encode_field(field);

    // let decode_sync = decoder_sync::prepare_decoder(field, &context);
    // let current_field_ref = match &field.type_ {
    //     Type::Container(_) => quote! { self },
    //     Type::Enum(_) => quote! { self },
    //     _ => quote! { self.0 },
    // };
    let mut decode_context = coder::decode::Context::new();
    decode_context.decode_field_top(field);
    let decode_sync = decoder_sync::prepare_decoder(&decode_context);

    let mut new_context = coder::encode::Context::new();
    new_context.encode_field_top(field);

    let encode_sync = encoder_sync::prepare_encoder(&new_context);
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

    quote! {
        impl #container_ident {
            pub fn decode_sync<R: Read + BufRead>(reader: &mut R #arguments) -> Result<Self> {
                #redefaults
                #decode_sync
            }

            // pub fn decode_buf<B: AsRef<[u8]>(mut buf: B) -> Self {
            //     unimplemented!()
            // }

            // pub async fn decode<R: AsyncReadRead>(mut reader: R) -> Self {
            //     unimplemented!()
            // }

            pub fn encode_sync<W: Write>(&self, writer: &mut W #arguments) -> Result<()> {
                #redefaults
                #encode_sync
            }
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

pub fn generate_container(
    name: &str,
    item: &ContainerType,
    options: &CompileOptions,
) -> TokenStream {
    let mut fields = vec![];
    for (name, field) in item.flatten_view() {
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
            pub #name_ident: #type_ref,
        });
    }
    let fields = flatten(fields);
    let name_ident = format_ident!("{}", global_name(name));
    let derives = options.emit_derives(&[]);

    quote! {
        #derives
        pub struct #name_ident {
            #fields
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
    let derives = options.emit_derives(&["Clone", "Copy"]);

    let format_string = format!("illegal enum value '{{}}' for enum '{}'", name);

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
                    x => Err(DecodeError(format!(#format_string, x)).into()),
                }
            }
        }
    }
}

fn emit_arguments<F: Fn(&Arc<Field>) -> TokenStream>(
    arguments: &[Expression],
    transform_arguments: &[TransformArgument],
    ref_resolver: &F,
) -> TokenStream {
    let mut args_emitted = vec![];
    let mut arguments = arguments.iter();
    for arg in transform_arguments.iter() {
        let value = arguments.next().map(|x| emit_expression(x, ref_resolver));
        let value = if arg.optional {
            if let Some(value) = value {
                quote! { Some(#value) }
            } else {
                quote! { None }
            }
        } else {
            value.unwrap()
        };
        let name = emit_ident(&arg.name);
        args_emitted.push(quote! {
            let #name = #value;
        });
    }
    flatten(args_emitted)
}
