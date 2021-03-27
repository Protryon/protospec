use crate::{asg::*};
use super::expr::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{sync::Arc, unimplemented};
use crate::coder::*;
use super::{ flatten, emit_ident, global_name, emit_arguments };

fn ref_resolver(f: &Arc<Field>) -> TokenStream {
    let f = format_ident!("f_{}", &f.name);
    quote! { #f }
}

fn prepare_decode_sync(context: &Context) -> TokenStream {
    let mut statements = vec![];
    for instruction in context.instructions.iter() {
        match instruction {
            Instruction::CodeRef(inner_field, arguments) => {
                let mut out_arguments = vec![];
                for argument in arguments {
                    let value = emit_expression(argument, &ref_resolver);
                    out_arguments.push(quote! {, #value});
                }
                let out_arguments = flatten(out_arguments);
                //todo: pass args to foreign
                match &inner_field.type_ {
                    Type::Foreign(foreign) => statements.push(foreign.obj.decoding_sync_gen()),
                    _ => {
                        let ref_ident = emit_ident(&inner_field.name);
                        statements.push(quote! {
                            { #ref_ident::decode_sync(reader #out_arguments)? }
                        })
                    }
                }
            }
            Instruction::CodePrimitive(PrimitiveType::Bool) => {
                statements.push(quote! {
                    {
                        let mut scratch = [0u8; 1];
                        reader.read_exact(&mut scratch[..1])?;
                        scratch[0] != 0
                    }
                });
            }
            Instruction::CodePrimitive(type_) => {
                let length = type_.size() as usize;

                statements.push(quote! {
                    {
                        let mut scratch = [0u8; 16];
                        reader.read_exact(&mut scratch[..#length])?;
                        #type_::from_be_bytes((&scratch[0..#length]).try_into()?)
                    }
                });
            }
            Instruction::CodeEnum(field) => {
                let type_ = match &field.type_ {
                    Type::Enum(t) => t,
                    _ => unimplemented!(),
                };
                let enum_ident = format_ident!("{}", &field.name);
                let rep = format_ident!("{}", type_.rep.to_string());
                let length = type_.rep.size() as usize;

                statements.push(quote! {
                    {
                        let mut scratch = [0u8; 16];
                        reader.read_exact(&mut scratch[..#length])?;
                        #enum_ident::from_repr(#rep::from_be_bytes((&scratch[0..#length]).try_into()?))?
                    }
                });
            }
            Instruction::CodePrimitiveArray(type_, length) => {
                let count = emit_expression(length, &ref_resolver);

                statements.push(quote! {
                    {
                        let t_count = #count as usize;
                        let mut t: Vec<#type_> = vec![0; t_count];
                        let t_borrow = &mut t[..];
                        let t_borrow2 = unsafe {
                            let len = t_borrow.len() * mem::size_of::<#type_>();
                            let ptr = t.as_ptr() as *mut u8;
                            slice::from_raw_parts_mut(ptr, len)
                        };
                        reader.read_exact(&mut t_borrow2[..])?;
                        t
                    }
                });
            }
            Instruction::CodeContainer(target, fields) => {
                let name = global_name(&target.name);
                let name_ident = emit_ident(&name);
                let mut field_tokens = vec![];
                let mut pre_tokens = vec![];
                if target.toplevel {
                    let type_ = match &target.type_ {
                        Type::Container(c) => c,
                        _ => unimplemented!(),
                    };

                    for (_, field) in type_.flatten_view() {
                        if type_.items.contains_key(&field.name) {
                            continue;
                        }
                        let field_ident_temp = format_ident!("f_{}", &field.name);
                        pre_tokens.push(quote! {
                            let #field_ident_temp;
                        });
                    }

                    for (field, context) in fields.iter() {
                        let ast = prepare_decode_sync(context);
                        if matches!(&field.type_, Type::Container(_)) {
                            pre_tokens.push(quote! {
                                #ast
                            });
                            continue;
                        }
                        let field_ident = emit_ident(&field.name);
                        let field_ident_temp = format_ident!("f_{}", &field.name);
                        pre_tokens.push(quote! {
                            let #field_ident_temp = #ast;
                        });
                        field_tokens.push(quote! {
                            #field_ident: #field_ident_temp,
                        });
                    }
                    for (_, field) in type_.flatten_view() {
                        if type_.items.contains_key(&field.name) {
                            continue;
                        }
                        let field_ident = emit_ident(&field.name);
                        let field_ident_temp = format_ident!("f_{}", &field.name);
                        field_tokens.push(quote! {
                            #field_ident: #field_ident_temp,
                        });
                    }
                    let pre_tokens = flatten(pre_tokens);
                    let field_tokens = flatten(field_tokens);
    
                    statements.push(quote! {
                        {
                            #pre_tokens
                            #name_ident {
                                #field_tokens
                            }
                        }
                    });
                } else {
                    for (field, context) in fields.iter() {
                        let ast = prepare_decode_sync(context);
                        if matches!(&field.type_, Type::Container(_)) {
                            pre_tokens.push(quote! {
                                #ast
                            });
                            continue;
                        }
                        let field_ident_temp = format_ident!("f_{}", &field.name);
                        pre_tokens.push(quote! {
                            #field_ident_temp = #ast;
                        });
                    }
                    let pre_tokens = flatten(pre_tokens);
    
                    statements.push(quote! {
                        {
                            #pre_tokens
                        }
                    });
                }
                
                
            }
            Instruction::CodeField(target, inner) => {
                let inner_emitted = prepare_decode_sync(&inner);
                let is_newtype =
                    !matches!(target.type_, Type::Container(_) | Type::Enum(_)) && target.toplevel;
                if is_newtype {
                    let target_ident = format_ident!("{}", target.name);
                    statements.push(quote! {
                        #target_ident({
                            #inner_emitted
                        })
                    });
                } else {
                    statements.push(quote! {
                        {
                            #inner_emitted
                        }
                    });
                }
                // #t_ident
            }
            Instruction::Bounded(inner, length) => {
                let length = emit_expression(length, &ref_resolver);
                let inner_emitted = prepare_decode_sync(&inner);

                statements.push(quote! {
                    {
                        let length = (#length) as usize;

                        let mut buf: Vec<u8> = Vec::with_capacity(length as usize);
                        unsafe { buf.set_len(length); }

                        reader.read_exact(&mut buf[..])?;
                        {
                            let mut reader = Cursor::new(buf);
                            let reader = &mut reader;
                            #inner_emitted
                        }
                    }
                });
            }
            Instruction::Repeat(inner, length) => {
                let length = emit_expression(length, &ref_resolver);
                let inner_emitted = prepare_decode_sync(&inner);

                statements.push(quote! {
                    {
                        let length = #length;

                        let mut out = Vec::with_capacity(length as usize);

                        for i in 0..length {
                            out.push(#inner_emitted);
                        }
                        out
                    }
                });
            }
            Instruction::RepeatUntilTerminator(inner, Some(terminator)) => {
                // let terminator = emit_expression(terminator);
                // let inner_emitted = prepare_decode_sync(&inner);

                // statements.push(quote! {
                //     let terminator = #terminator;

                //     let mut out = vec![];

                //     loop {
                //         #inner_emitted
                //         out.push(t);
                //     }
                //     let t = out;
                // });
                unimplemented!();
            }
            Instruction::RepeatUntilTerminator(inner, None) => {
                let inner_emitted = prepare_decode_sync(&inner);
                //todo: optimize
                statements.push(quote! {
                    {
                        let mut r = vec![];
                        reader.read_to_end(&mut r)?;
                        let r_len = r.len() as u64;

                        let mut out = vec![];
                        {
                            let mut reader = Cursor::new(r);
                            let reader = &mut reader;
                            while reader.position() < r_len {
                                out.push(#inner_emitted);
                            }
                        }
                        out
                    }
                });
            }
            Instruction::If(condition, inner) => {
                let condition = emit_expression(condition, &ref_resolver);
                let inner_emitted = prepare_decode_sync(&inner);

                statements.push(quote! {
                    { if #condition {
                        Some(#inner_emitted)
                    } else {
                        None
                    } }
                });
            }
            Instruction::Transform(transform, arguments, inner) => {
                let inner_emitted = prepare_decode_sync(&inner);
                let transform_emitted = transform.inner.decoding_sync_gen();
                let args_emitted =
                    emit_arguments(&arguments[..], &transform.arguments[..], &ref_resolver);

                statements.push(quote! {
                    {

                        let mut reader = {
                            #args_emitted
                            #transform_emitted
                        };
                        let reader = &mut reader;
                        #inner_emitted
                    }
                });
            }
            Instruction::TransformIf(condition, transform, arguments, inner) => {
                let condition = emit_expression(condition, &ref_resolver);
                let inner_emitted = prepare_decode_sync(&inner);
                let transform_emitted = transform.inner.decoding_sync_gen();
                let args_emitted =
                    emit_arguments(&arguments[..], &transform.arguments[..], &ref_resolver);

                statements.push(quote! {
                    {
                        if #condition {
                            let mut reader = {
                                #args_emitted
                                #transform_emitted
                            };
                            let reader = &mut reader;
                            #inner_emitted
                        } else {
                            #inner_emitted
                        }
                    }
                });
            }
        }
    }

    let statements = flatten(statements);
    quote! {
        #statements
    }
}

pub fn prepare_decoder(target: &Arc<Field>, context: &Context) -> TokenStream {
    // let field_ident = format_ident!("f_{}", target.name);
    let decode_sync = prepare_decode_sync(&context);
    // let field_constructor = match &target.type_ {
    //     Type::Container(_) | Type::Enum(_) => quote! { #field_ident },
    //     _ => {
    //         let container_ident = format_ident!("{}", global_name(&target.name));
    //         quote! { #container_ident(#field_ident) }
    //     }
    // };
    //todo: trait
    quote! {
        Ok(#decode_sync)

    } // Ok(#field_constructor)
}
