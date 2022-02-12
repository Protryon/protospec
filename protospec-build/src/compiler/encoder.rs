use super::*;
use crate::coder::encode::*;
use crate::{coder::*, map_async};

fn ref_resolver(f: &Arc<Field>) -> TokenStream {
    let f = emit_ident(&f.name);
    quote! { self.#f }
}

fn emit_target(target: &Target) -> TokenStream {
    match target {
        Target::Direct => quote! { writer },
        Target::Stream(x) => emit_register(*x),
        Target::Buf(x) => {
            let buf = emit_register(*x);
            quote! { (&mut #buf) }
        }
    }
}

fn prepare_encode(instructions: &[Instruction], is_async: bool, is_root: bool) -> TokenStream {
    let async_ = map_async(is_async);
    let mut statements = vec![];
    if is_root {
        if is_async {
            statements.push(quote! {
                use tokio::io::{ AsyncWrite, AsyncWriteExt };
            })
        } else {
            statements.push(quote! {
                use std::io::Write;
            })
        }
    }
    for instruction in instructions.iter() {
        match instruction {
            Instruction::Eval(target, expr) => {
                let target = emit_register(*target);
                let value = emit_expression(expr, &ref_resolver);
                statements.push(quote! {
                    let #target = #value;
                });
            }
            Instruction::GetField(target, source, op) => {
                let target = emit_register(*target);
                let mut source = emit_register(*source);
                for op in op.iter() {
                    source = match &op {
                        FieldRef::Name(name) => {
                            let name = format_ident!("{}", name);
                            quote! { #source.#name }
                        }
                        FieldRef::ArrayAccess(index) => {
                            let index = emit_register(*index);
                            quote! { #source[#index as usize] }
                        }
                        FieldRef::TupleAccess(x) => {
                            quote! { #source.#x }
                        }
                    };
                }
                statements.push(quote! {
                    let #target = &#source;
                });
            }
            Instruction::AllocBuf(buf, len) => {
                let buf = emit_register(*buf);
                let len = emit_register(*len);
                statements.push(quote! {
                    //todo: strictly bound this
                    let mut #buf: Vec<u8> = Vec::with_capacity(#len as usize);
                });
            }
            Instruction::AllocDynBuf(buf) => {
                let buf = emit_register(*buf);
                statements.push(quote! {
                    let mut #buf: Vec<u8> = Vec::new();
                });
            }
            Instruction::ProxyStream(stream, new_stream) => {
                let new_stream_value = emit_register(*new_stream);
                let input = emit_target(stream);
                statements.push(quote! {
                    let mut #new_stream_value = #input;
                });
            }
            Instruction::Loop(index, stop_index, inner) => {
                let index = emit_register(*index);
                let inner = prepare_encode(&inner[..], is_async, false);
                let stop = emit_register(*stop_index);
                statements.push(quote! {
                    for #index in 0..#stop {
                        #inner
                    }
                });
            }
            Instruction::GetLen(len, source, cast_type) => {
                let len = emit_register(*len);
                let source = emit_register(*source);
                let cast = if let Some(cast_type) = cast_type {
                    let cast_type = emit_ident(&cast_type.to_string());
                    quote! {
                        as #cast_type
                    }
                } else {
                    quote! {}
                };
                statements.push(quote! {
                    let #len = #source.len() #cast;
                });
            }
            Instruction::NullCheck(target, destination, message) => {
                let target = emit_register(*target);
                let destination = emit_register(*destination);

                statements.push(quote! {
                    let #destination = if let Some(#destination) = &#target {
                        #destination
                    } else {
                        return Err(EncodeError(#message.to_string()).into())
                    };
                });
            }
            Instruction::Conditional(condition, if_true, if_false) => {
                let condition = emit_register(*condition);
                let if_true = prepare_encode(&if_true[..], is_async, false);
                let if_false = prepare_encode(&if_false[..], is_async, false);
                statements.push(quote! {
                    if #condition {
                        #if_true
                    } else {
                        #if_false
                    }
                });
            }
            Instruction::Drop(register) => {
                let register = emit_register(*register);
                statements.push(quote! {
                    drop(#register);
                });
            }
            Instruction::WrapStream(stream, new_stream, transformer, args) => {
                let new_stream_value = emit_register(*new_stream);
                let args = args.iter().map(|x| emit_register(*x)).collect::<Vec<_>>();
                let input = emit_target(stream);
                let transformed = transformer.inner.encoding_gen(input, args, is_async);
                statements.push(quote! {
                    let mut #new_stream_value = #transformed;
                })
            }
            Instruction::EncodeForeign(target, data, type_ref, args) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                let mut out_arguments = vec![];
                for argument in args {
                    let value = emit_register(*argument);
                    out_arguments.push(value);
                }

                statements.push(
                    type_ref
                        .obj
                        .encoding_gen(target, data, out_arguments, is_async),
                );
            }
            Instruction::EndStream(stream) => {
                let stream = emit_register(*stream);
                statements.push(quote! {
                    #stream.flush()#async_?;
                    drop(#stream);
                });
            }
            Instruction::EmitBuf(target, buf) => {
                let target = emit_target(target);
                let buf = emit_register(*buf);
                statements.push(quote! {
                    #target.write_all(&#buf[..])#async_?;
                });
            }
            Instruction::EncodeRef(target, source, args) => {
                let mut out_arguments = vec![];
                for argument in args {
                    let value = emit_register(*argument);
                    out_arguments.push(quote! {, #value});
                }
                let out_arguments = flatten(out_arguments);
                let target = emit_target(target);
                let source = emit_register(*source);
                if is_async {
                    statements.push(quote! {
                        #source.encode_async(#target #out_arguments).await?;
                    });
                } else {
                    statements.push(quote! {
                        #source.encode_sync(#target #out_arguments)?;
                    });
                }
            }
            Instruction::EncodeEnum(type_, target, value) => {
                let target = emit_target(target);
                let value = emit_register(*value);
                statements.push(quote! {
                    #target.write_all(&(*#value as #type_).to_be_bytes()[..])#async_?;
                });
            }
            Instruction::EncodePrimitive(target, data, PrimitiveType::Bool) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                statements.push(quote! {
                    #target.write_all(&[if *#data { 1u8 } else { 0u8 }])#async_?;
                });
            }
            Instruction::EncodePrimitive(target, data, _) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                statements.push(quote! {
                    #target.write_all(&#data.to_be_bytes()[..])#async_?;
                });
            }
            Instruction::EncodePrimitiveArray(target, data, type_, len) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                if let Some(len) = len {
                    let len = emit_register(*len);
                    statements.push(quote! {
                        {
                            let t_count = #len as usize;
                            if t_count != #data.len() {
                                //todo: throw an error properly
                                assert_eq!(t_count, #data.len());
                            }
                            let t_borrow = &#data[..];
                            let t_borrow2 = unsafe {
                                let len = t_borrow.len() * mem::size_of::<#type_>();
                                let ptr = #data.as_ptr() as *const u8;
                                slice::from_raw_parts(ptr, len)
                            };
                            #target.write_all(&t_borrow2[..])#async_?;
                        }
                    });
                } else {
                    statements.push(quote! {
                        {
                            let t_borrow = &#data[..];
                            let t_borrow2 = unsafe {
                                let len = t_borrow.len() * mem::size_of::<#type_>();
                                let ptr = #data.as_ptr() as *const u8;
                                slice::from_raw_parts(ptr, len)
                            };
                            #target.write_all(&t_borrow2[..])#async_?;
                        }
                    });
                }
            }
            Instruction::ConditionalWrapStream(
                condition,
                prelude,
                stream,
                new_stream,
                owned_new_stream,
                transformer,
                args,
            ) => {
                let condition = emit_register(*condition);
                let new_stream_value = emit_register(*new_stream);
                let owned_new_stream = emit_register(*owned_new_stream);
                let args = args.iter().map(|x| emit_register(*x)).collect::<Vec<_>>();
                let input = emit_target(stream);
                let transformed = transformer.inner.encoding_gen(input.clone(), args, is_async);
                let prelude = prepare_encode(&prelude[..], is_async, false);
    
                let trait_name = if is_async {
                    quote! { dyn AsyncWrite + Send + Sync + Unpin }
                } else {
                    quote! { dyn Write }
                };

                //todo: would be nicer to use generics here instead of trait object
                statements.push(quote! {
                    let mut #owned_new_stream = None;
                    let #new_stream_value: &mut #trait_name = if #condition {
                        #prelude
                        #owned_new_stream = Some(#transformed);
                        #owned_new_stream.as_mut().unwrap()
                    } else {
                        #input as &mut #trait_name
                    };
                })
            }
            Instruction::UnwrapEnum(enum_name, discriminant, original, checked, message) => {
                let enum_name = emit_ident(enum_name);
                let discriminant = emit_ident(discriminant);
                let original = emit_register(*original);
                let checked = emit_register(*checked);

                statements.push(quote! {
                    let #checked = if let #enum_name::#discriminant(#checked) = &#original {
                        #checked
                    } else {
                        return Err(EncodeError(#message.to_string()).into())
                    };
                });
            },
            Instruction::UnwrapEnumStruct(enum_name, discriminant, original, checked, message) => {
                let enum_name = emit_ident(enum_name);
                let discriminant = emit_ident(discriminant);
                let original = emit_register(*original);
                // let checked = emit_register(*checked);
                let mut checked_name_list = quote! {};
                let mut checked_reg_list = quote! {};
                for (name, checked) in checked.iter().rev() {
                    let name = emit_ident(name);
                    let checked = emit_register(*checked);
                    checked_name_list = quote! { #name: #checked, #checked_name_list };
                    checked_reg_list = quote! { #checked, #checked_reg_list };
                }

                statements.push(quote! {
                    let (#checked_reg_list) = if let #enum_name::#discriminant { #checked_name_list } = &#original {
                        (#checked_reg_list)
                    } else {
                        return Err(EncodeError(#message.to_string()).into())
                    };
                });
            },
            Instruction::BreakBlock(instructions) => {
                //todo: support nested breakblocks?
                let interior = prepare_encode(&instructions[..], is_async, false);
                statements.push(quote! {
                    'bb: loop {
                        #interior
                        break 'bb;
                    }
                });
            }, 
            Instruction::Break => {
                statements.push(quote! {
                    break 'bb;
                });
            },    
        }
    }

    let statements = flatten(statements);
    quote! {
        #statements
    }
}

pub fn prepare_encoder(coder: &Context, is_async: bool) -> TokenStream {
    let decode_sync = prepare_encode(&coder.instructions[..], is_async, true);
    let base = emit_register(0);
    quote! {
        let #base = self;
        #decode_sync
        Ok(())
    }
}
