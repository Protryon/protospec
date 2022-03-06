use super::*;
use crate::coder::decode::*;
use crate::{coder::*, map_async};

fn emit_target(target: &Target) -> TokenStream {
    match target {
        Target::Direct => quote! { reader },
        Target::Stream(x) => emit_register(*x),
        Target::Buf(x) => {
            let buf = emit_register(*x);
            quote! { (&mut #buf) }
        }
    }
}

fn prepare_decode(
    options: &CompileOptions,
    context: &Context,
    instructions: &[Instruction],
    is_async: bool,
    is_root: bool,
) -> TokenStream {
    let async_ = map_async(is_async);
    let mut statements = vec![];
    if is_root {
        if is_async {
            statements.push(quote! {
                use tokio::io::{ AsyncRead, AsyncBufRead, AsyncBufReadExt, AsyncReadExt };
            })
        } else {
            statements.push(quote! {
                use std::io::Read;
            })
        }
    }

    for instruction in instructions.iter() {
        if options.debug_mode {
            let raw = format!("decode {}: {:?}", context.name, instruction);
            statements.push(quote! {
                println!("{}", #raw);
            });
        }
        match instruction {
            Instruction::Eval(target, expr, field_register_map) => {
                let target = emit_register(*target);
                let value = emit_expression(expr, &|field| {
                    emit_register(
                        *field_register_map
                            .get(&field.name)
                            .expect("missing register for field"),
                    )
                });
                statements.push(quote! {
                    let #target = #value;
                });
            }
            Instruction::Construct(target, Constructable::Tuple(items)) => {
                let target = emit_register(*target);
                let items = flatten(
                    items
                        .iter()
                        .map(|x| {
                            let x = emit_register(*x);
                            quote! {#x, }
                        })
                        .collect::<Vec<_>>(),
                );
                statements.push(quote! {
                    let #target = (#items);
                });
            }
            Instruction::Construct(target, Constructable::TaggedTuple { name, items }) => {
                let target = emit_register(*target);
                let items = flatten(
                    items
                        .iter()
                        .map(|x| {
                            let x = emit_register(*x);
                            quote! {#x, }
                        })
                        .collect::<Vec<_>>(),
                );
                let name = emit_ident(name);
                statements.push(quote! {
                    let #target = #name(#items);
                });
            }
            Instruction::Construct(target, Constructable::Struct { name, items }) => {
                let target = emit_register(*target);
                let items = flatten(
                    items
                        .iter()
                        .map(|(name, x)| {
                            let x = emit_register(*x);
                            let name = emit_ident(name);
                            quote! {#name: #x,}
                        })
                        .collect::<Vec<_>>(),
                );
                let name = emit_ident(name);
                statements.push(quote! {
                    let #target = #name { #items };
                });
            }
            Instruction::Construct(target, Constructable::TaggedEnum { name, discriminant, values }) => {
                let target = emit_register(*target);
                let items = flatten(
                    values
                        .iter()
                        .map(|x| {
                            let x = emit_register(*x);
                            quote! {#x, }
                        })
                        .collect::<Vec<_>>(),
                );
                let name = emit_ident(name);
                let discriminant = emit_ident(discriminant);
                statements.push(quote! {
                    let #target = #name::#discriminant(#items);
                });
            }
            Instruction::Construct(target, Constructable::TaggedEnumStruct { name, discriminant, values }) => {
                let target = emit_register(*target);
                let items = flatten(
                    values
                        .iter()
                        .map(|(name, x)| {
                            let x = emit_register(*x);
                            let name = emit_ident(name);
                            quote! {#name: #x,}
                        })
                        .collect::<Vec<_>>(),
                );
                let name = emit_ident(name);
                let discriminant = emit_ident(discriminant);
                statements.push(quote! {
                    let #target = #name::#discriminant { #items };
                });
            }
            Instruction::Constrict(stream, new_stream, len) => {
                let stream = emit_target(stream);
                let new_stream = emit_register(*new_stream);
                let len = emit_register(*len);
                statements.push(quote! {
                    let mut #new_stream = #stream.take(#len as u64);
                    let #new_stream = &mut #new_stream;
                });
            }
            Instruction::WrapStream(stream, new_stream, transformer, args) => {
                let new_stream_value = emit_register(*new_stream);
                let args = args.iter().map(|x| emit_register(*x)).collect::<Vec<_>>();
                let input = emit_target(stream);
                let transformed = transformer.inner.decoding_gen(input, args, is_async);
                statements.push(quote! {
                    let mut #new_stream_value = #transformed;
                    let #new_stream_value = &mut #new_stream_value;
                })
            }
            Instruction::ConditionalWrapStream(
                condition,
                prelude,
                stream,
                new_stream,
                transformer,
                args,
            ) => {
                let condition = emit_register(*condition);
                let new_stream_value = emit_register(*new_stream);
                let args = args.iter().map(|x| emit_register(*x)).collect::<Vec<_>>();
                let input = emit_target(stream);
                let transformed = transformer
                    .inner
                    .decoding_gen(input.clone(), args, is_async);
                let prelude = prepare_decode(options, context, &prelude[..], is_async, false);

                //todo: would be nicer to use generics here instead of trait object
                if is_async {
                    statements.push(quote! {
                        let mut r_xform;
                        let #new_stream_value: &mut dyn AsyncBufRead + Unpin + Send + Sync = if #condition {
                            #prelude
                            r_xform = #transformed;
                            &mut r_xform
                        } else {
                            #input as &mut dyn AsyncBufRead + Unpin + Send + Sync
                        };
                    })
                } else {
                    statements.push(quote! {
                        let mut r_xform;
                        let #new_stream_value: &mut dyn Read = if #condition {
                            #prelude
                            r_xform = #transformed;
                            &mut r_xform
                        } else {
                            #input as &mut dyn Read
                        };
                    })
                }
            }
            Instruction::DecodeForeign(target, data, type_ref, args) => {
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
                        .decoding_gen(target, data, out_arguments, is_async),
                );
            }
            Instruction::DecodeRef(target, source, class, args) => {
                let mut out_arguments = vec![];
                for argument in args {
                    let value = emit_register(*argument);
                    out_arguments.push(quote! {, #value});
                }
                let out_arguments = flatten(out_arguments);
                let target = emit_target(target);
                let source = emit_register(*source);
                let class = emit_ident(class);
                if is_async {
                    statements.push(quote! {
                        let #source = #class::decode_async(#target #out_arguments).await?;
                    });
                } else {
                    statements.push(quote! {
                        let #source = #class::decode_sync(#target #out_arguments)?;
                    });
                }
            }
            Instruction::DecodeRepr(name, type_, value, target) => {
                let target = emit_target(target);
                let value = emit_register(*value);

                let enum_ident = format_ident!("{}", &name);
                let length = type_.size() as usize;

                statements.push(quote! {
                    let #value = {
                        let mut scratch = [0u8; #length];
                        #target.read_exact(&mut scratch[..])#async_?;
                        #enum_ident::from_repr(#type_::from_be_bytes((&scratch[..]).try_into()?))?
                    };
                });
            }
            Instruction::DecodePrimitive(target, data, PrimitiveType::Bool) => {
                let target = emit_target(target);
                let data = emit_register(*data);

                statements.push(quote! {
                    let #data = {
                        let mut scratch = [0u8; 1];
                        #target.read_exact(&mut scratch[..1])#async_?;
                        scratch[0] != 0
                    };
                });
            }
            Instruction::DecodePrimitive(target, data, type_) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                let length = type_.size() as usize;

                statements.push(quote! {
                    let #data = {
                        let mut scratch = [0u8; #length];
                        #target.read_exact(&mut scratch[..])#async_?;
                        #type_::from_be_bytes((&scratch[..]).try_into()?)
                    };
                });
            }
            Instruction::DecodePrimitiveArray(target, data, type_, len) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                if let Some(len) = len {
                    let len = emit_register(*len);
                    statements.push(quote! {
                        let #data = {
                            let t_count = #len as usize;
                            let size = mem::size_of::<#type_>();
                            let mut raw: Vec<u8> = Vec::with_capacity(t_count * size);
                            unsafe { raw.set_len(t_count * size) };
                            #target.read_exact(&mut raw[..])#async_?;
                            raw.chunks_exact(size).map(|x| #type_::from_be_bytes(x.try_into().unwrap())).collect::<Vec<#type_>>()
                        };
                    });
                } else {
                    statements.push(quote! {
                        let #data = {
                            let mut raw: Vec<u8> = Vec::new();
                            #target.read_to_end(&mut raw)#async_?;
                            let size = mem::size_of::<#type_>();
                            raw.chunks_exact(size).map(|x| #type_::from_be_bytes(x.try_into().unwrap())).collect::<Vec<#type_>>()
                        };
                    });
                }
            }
            Instruction::DecodeReprArray(target, data, name, type_, len) => {
                let target = emit_target(target);
                let data = emit_register(*data);
                let enum_ident = format_ident!("{}", &name);
                if let Some(len) = len {
                    let len = emit_register(*len);
                    statements.push(quote! {
                        let #data = {
                            let t_count = #len as usize;
                            let size = mem::size_of::<#type_>();
                            let mut raw: Vec<u8> = Vec::with_capacity(t_count * size);
                            unsafe { raw.set_len(t_count * size) };
                            #target.read_exact(&mut raw[..])#async_?;
                            raw.chunks_exact(size).map(|x| #enum_ident::from_repr(#type_::from_be_bytes(x.try_into().unwrap()))).collect::<Result<Vec<#enum_ident>>>()?
                        };
                    });
                } else {
                    statements.push(quote! {
                        let #data = {
                            let mut raw: Vec<u8> = Vec::new();
                            #target.read_to_end(&mut raw)#async_?;
                            let size = mem::size_of::<#type_>();
                            raw.chunks_exact(size).map(|x| #enum_ident::from_repr(#type_::from_be_bytes(x.try_into().unwrap()))).collect::<Result<Vec<#enum_ident>>>()?
                        };
                    });
                }
            }
            Instruction::Loop(target, stop_index, terminator, output, inner) => {
                let output = emit_register(*output);
                let inner = prepare_decode(options, context, &inner[..], is_async, false);
                let stop = stop_index.map(emit_register);
                let terminator = terminator.map(emit_register);
                let target = emit_target(target);
                if let Some(stop) = stop {
                    statements.push(quote! {
                        let mut #output = Vec::with_capacity(#stop as usize);
                        for _ in 0..#stop {
                            #inner
                        }
                    });
                } else if let Some(terminator) = terminator {
                    statements.push(quote! {
                        let mut #output = Vec::new();
                        loop {
                            let buf = #target.fill_buf()#async_?;
                            if buf.len() == 0 {
                                break;
                            }
                            if (buf.len() < #terminator.len()) {
                                //todo: confirm this cannot infinite loop
                                continue;
                            }
                            if &buf[..#terminator.len()] == #terminator {
                                #target.consume(#terminator.len());
                                break;
                            }
                            #inner
                        }
                    });
                } else {
                    statements.push(quote! {
                        let mut #output = Vec::new();
                        //TODO: optimize this to not buffer with a Peekable type
                        {
                            let mut r = vec![];
                            #target.read_to_end(&mut r)#async_?;
                            let r_len = r.len() as u64;

                            {
                                let mut #target = Cursor::new(r);
                                let #target = &mut #target;
                                while #target.position() < r_len {
                                    #inner
                                }
                            }
                        }
                    });
                }
            }
            Instruction::LoopOutput(output, item) => {
                let output = emit_register(*output);
                let item = emit_register(*item);
                statements.push(quote! {
                    #output.push(#item);
                });
            }
            Instruction::Conditional(target, interior, condition, inner) => {
                let targets = target.iter().copied().map(emit_register).collect::<Vec<_>>();
                let targets = flatten_separated(targets, quote! {,});
                let targets = if target.len() > 1 {
                    quote! { (#targets) }
                } else {
                    targets
                };
                let interiors = interior.iter().copied().map(|r| {
                    let r = emit_register(r);
                    quote! {
                        Some(#r)
                    }
                }).collect::<Vec<_>>();
                let anti_interiors = interior.iter().map(|_| quote! { None }).collect::<Vec<_>>();

                let interiors = flatten_separated(interiors, quote! {,});
                let interiors = if interior.len() > 1 {
                    quote! { (#interiors) }
                } else {
                    interiors
                };

                let anti_interiors = flatten_separated(anti_interiors, quote! {,});
                let anti_interiors = if interior.len() > 1 {
                    quote! { (#anti_interiors) }
                } else {
                    anti_interiors
                };

                let condition = emit_register(*condition);
                let inner = prepare_decode(options, context, &inner[..], is_async, false);
                statements.push(quote! {
                    let #targets = if #condition {
                        #inner
                        #interiors
                    } else {
                        #anti_interiors
                    };
                });
            }
            Instruction::ConditionalPredicate(condition, inner) => {
                let condition = emit_register(*condition);
                let inner = prepare_decode(options, context, &inner[..], is_async, false);
                statements.push(quote! {
                    if #condition {
                        #inner
                    }
                });
            },
            Instruction::Return(result) => {
                let result = emit_register(*result);
                statements.push(quote! {
                    return Ok(#result);
                });
            },
            Instruction::Error(e) => {
                statements.push(quote! {
                    return Err(decode_error(#e).into());
                });
            },
            Instruction::Skip(target, len) => {
                let target = emit_target(target);
                let len = emit_register(*len);
                statements.push(quote! {
                    let mut big_scratch = vec![0u8; #len as usize];
                    #target.read_exact(&mut big_scratch[..])#async_?;
                });
            },
        }
    }

    let statements = flatten(statements);
    quote! {
        #statements
    }
}

pub fn prepare_decoder(options: &CompileOptions, coder: &Context, is_async: bool) -> TokenStream {
    let decode = prepare_decode(options, &coder, &coder.instructions[..], is_async, true);
    quote! {
        #decode
    }
}
