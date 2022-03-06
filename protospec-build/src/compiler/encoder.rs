use std::collections::HashMap;

use super::*;
use crate::coder::encode::*;
use crate::{coder::*, map_async, ScalarType};

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

struct EncoderContext {
    is_async: bool,
    resolved_refs: HashMap<String, usize>,
}
impl EncoderContext {
    fn prepare_encode(&mut self, instructions: &[Instruction], is_root: bool) -> TokenStream {
        let async_ = map_async(self.is_async);
        let mut statements = vec![];
        if is_root {
            if self.is_async {
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
            // println!("encoding {}", instruction);
            match instruction {
                Instruction::Eval(target, expr) => {
                    let target = emit_register(*target);
                    let value = emit_expression(expr, &|f: &Arc<Field>| {
                        let register = self.resolved_refs.get(&*f.name).expect("failed to dereference");
                        emit_register(*register)
                    });
                    statements.push(quote! {
                        let #target = #value;
                    });
                }
                Instruction::GetField(target, source, op) => {
                    let target = emit_register(*target);
                    let mut source = emit_register(*source);
                    for op in op.iter() {
                        source = match &op {
                            FieldRef::Ref => {
                                quote! { &#source }
                            }
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
                        let #target = #source;
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
                Instruction::Loop(index, stop_index, inner) => {
                    let index = emit_register(*index);
                    let inner = self.prepare_encode(&inner[..], false);
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
                Instruction::NullCheck(target, destination, is_copyable, message) => {
                    let target = emit_register(*target);
                    let destination = emit_register(*destination);
                    let ref_token = if *is_copyable {
                        quote! {}
                    } else {
                        quote! { & }
                    };
    
                    statements.push(quote! {
                        let #destination = if let Some(#destination) = #ref_token#target {
                            #destination
                        } else {
                            return Err(encode_error(#message).into())
                        };
                    });
                }
                Instruction::Conditional(condition, if_true, if_false) => {
                    let condition = emit_register(*condition);
                    let if_true = self.prepare_encode(&if_true[..], false);
                    let if_false = self.prepare_encode(&if_false[..], false);
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
                    let transformed = transformer.inner.encoding_gen(input, args, self.is_async);
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
                            .encoding_gen(target, data, out_arguments, self.is_async),
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
                    if self.is_async {
                        statements.push(quote! {
                            #source.encode_async(#target #out_arguments).await?;
                        });
                    } else {
                        statements.push(quote! {
                            #source.encode_sync(#target #out_arguments)?;
                        });
                    }
                }
                Instruction::EncodeEnum(target, value) => {
                    let target = emit_target(target);
                    let value = emit_register(*value);
                    statements.push(quote! {
                        #target.write_all(&((#value).to_repr()).to_be_bytes()[..])#async_?;
                    });
                }
                Instruction::EncodeBitfield(target, value) => {
                    let target = emit_target(target);
                    let value = emit_register(*value);
                    statements.push(quote! {
                        #target.write_all(&(#value.0).to_be_bytes()[..])#async_?;
                    });
                }
                Instruction::EncodePrimitive(target, data, PrimitiveType::Bool) => {
                    let target = emit_target(target);
                    let data = emit_register(*data);
                    statements.push(quote! {
                        #target.write_all(&[if #data { 1u8 } else { 0u8 }])#async_?;
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
                    let writing = match type_ {
                        Type::Bool => {
                            quote! {
                                for x in #data.iter() {
                                    #target.write_all(&[if x { 1u8 } else { 0u8 }])#async_?;
                                }
                            }
                        },
                        Type::Scalar(ScalarType::U8) => {
                            quote! {
                                #target.write_all(&#data[..])#async_?;
                            }
                        },
                        _ => {
                            quote! {
                                for x in #data.iter() {
                                    #target.write_all(&x.to_be_bytes()[..])#async_?;
                                }
                            }
                        },
                    };
                    if let Some(len) = len {
                        let len = emit_register(*len);
                        statements.push(quote! {
                            {
                                let t_count = #len as usize;
                                if t_count != #data.len() {
                                    //todo: throw an error properly
                                    assert_eq!(t_count, #data.len());
                                }
                                #writing
                            }
                        });
                    } else {
                        statements.push(quote! {
                            {
                                #writing
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
                    let transformed = transformer.inner.encoding_gen(input.clone(), args, self.is_async);
                    let prelude = self.prepare_encode(&prelude[..], false);
        
                    let trait_name = if self.is_async {
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
                            return Err(encode_error(#message).into())
                        };
                    });
                },
                Instruction::UnwrapEnumStruct(enum_name, discriminant, original, checked, message) => {
                    let enum_name = emit_ident(enum_name);
                    let discriminant = emit_ident(discriminant);
                    let original = emit_register(*original);
                    // let checked = emit_register(*checked);
                    let mut checked_name_list = quote! {};
                    let mut checked_reg_match = quote! {};
                    let mut checked_reg_list = quote! {};
                    for (name, checked, do_copy) in checked.iter().rev() {
                        let name = emit_ident(name);
                        let checked = emit_register(*checked);
                        checked_name_list = quote! { #name: #checked, #checked_name_list };
                        let copy = if *do_copy {
                            quote! { * }
                        } else {
                            quote! { }
                        };
                        checked_reg_match = quote! { #checked, #checked_reg_match };
                        checked_reg_list = quote! { #copy#checked, #checked_reg_list };
                    }
    
                    statements.push(quote! {
                        let (#checked_reg_match) = if let #enum_name::#discriminant { #checked_name_list } = &#original {
                            (#checked_reg_list)
                        } else {
                            return Err(encode_error(#message).into())
                        };
                    });
                },
                Instruction::BreakBlock(instructions) => {
                    //todo: support nested breakblocks?
                    let interior = self.prepare_encode(&instructions[..], false);
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
                Instruction::Pad(target, length) => {
                    let length = emit_register(*length);
                    let target = emit_target(target);
                    //todo: dont alloc on the heap
                    statements.push(quote! {
                        #target.write_all(&vec![0u8; #length as usize][..])#async_?;
                    });
                },
                Instruction::SetRef(name, value) => {
                    self.resolved_refs.insert(name.clone(), *value);
                },
                Instruction::GetRef(target, name) => {
                    let target = emit_register(*target);
                    let value = emit_register(*self.resolved_refs.get(name).expect("unresolved ref"));
                    statements.push(quote! {
                        let #target = #value;
                    });
                },
            }
        }
    
        let statements = flatten(statements);
        quote! {
            #statements
        }
    }
    
}

pub fn prepare_encoder(coder: &Context, is_async: bool) -> TokenStream {
    let mut context = EncoderContext {
        is_async,
        resolved_refs: Default::default(),
    };
    let decode_sync = context.prepare_encode(&coder.instructions[..], true);
    let base = emit_register(0);
    quote! {
        let #base = self;
        #decode_sync
        Ok(())
    }
}
