mod compiler;
mod parse;
mod semantic;
use std::collections::HashMap;

use indexmap::IndexMap;
use proc_macro2::TokenStream;
use protospec_build::{ffi::ForeignType, *};
use quote::quote;

pub fn load_asg(content: &str) -> AsgResult<asg::Program> {
    load_asg_with(content, TestImportResolver)
}

pub fn load_asg_with<T: ImportResolver + 'static>(
    content: &str,
    resolver: T,
) -> AsgResult<asg::Program> {
    let resolver = PreludeImportResolver(resolver);
    asg::Program::from_ast(
        &parse(content).map_err(|x| -> Error { x.into() })?,
        &resolver,
    )
}

#[derive(Debug)]
pub struct TestTransform;

#[derive(Debug)]
pub struct TestType;

impl ForeignTransform for TestTransform {
    fn decoding_gen(
        &self,
        input_stream: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        let offset = arguments.into_iter().next().unwrap_or_else(|| quote! { 1 });
        quote! {
            {
                let mut raw: Vec<u8> = Vec::new();
                #input_stream.read_to_end(&mut raw)?;
                Cursor::new(raw.iter().map(|x| x.wrapping_sub(#offset as u8)).collect::<Vec<u8>>())
            }
        }
    }

    fn arguments(&self) -> Vec<FFIArgument> {
        vec![FFIArgument {
            name: "offset".to_string(),
            type_: Some(asg::Type::Scalar(ScalarType::U8.into())),
            optional: true,
        }]
    }

    fn encoding_gen(
        &self,
        input_stream: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        let offset = arguments.into_iter().next().unwrap_or_else(|| quote! { 1 });
        quote! {
            {
                struct _X<'a, W: Write> {
                    inner: &'a mut W,
                    offset: u8,
                    buf: Vec<u8>,
                }
                impl<'a, W: Write> Write for _X<'a, W> {
                    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                        self.buf.extend_from_slice(buf);
                        Ok(buf.len())
                    }

                    fn flush(&mut self) -> std::io::Result<()> {
                        for i in 0..self.buf.len() {
                            let n = self.buf[i].wrapping_add(self.offset);
                            self.buf[i] = n;
                        }
                        self.inner.write_all(&self.buf[..])?;
                        self.buf.truncate(0);
                        Ok(())
                    }
                }
                _X {
                    inner: #input_stream,
                    offset: #offset as u8,
                    buf: vec![],
                }
            }
        }
    }
}

impl ForeignType for TestType {
    fn assignable_from(&self, type_: &asg::Type) -> bool {
        match type_ {
            asg::Type::Scalar(EndianScalarType {
                scalar: ScalarType::U32,
                ..
            }) => true,
            _ => false,
        }
    }

    fn assignable_to(&self, type_: &asg::Type) -> bool {
        match type_ {
            asg::Type::Scalar(EndianScalarType {
                scalar: ScalarType::U32,
                ..
            }) => true,
            _ => false,
        }
    }

    fn decoding_gen(
        &self,
        source: TokenStream,
        output_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        quote! {
            let #output_ref = Box::new({
                let mut scratch = [0u8; 4];
                #source.read_exact(&mut scratch[..])?;
                u32::from_be_bytes((&scratch[0..]).try_into()?)
            });
        }
    }

    fn encoding_gen(
        &self,
        target: TokenStream,
        field_ref: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        quote! {
            #target.write_all(&#field_ref.to_be_bytes()[..])?;
        }
    }

    fn type_ref(&self) -> TokenStream {
        quote! { Box<u32> }
    }

    fn arguments(&self) -> Vec<asg::TypeArgument> {
        vec![]
    }

    fn copyable(&self) -> bool {
        false
    }
}

pub struct TestImportResolver;

impl ImportResolver for TestImportResolver {
    fn normalize_import(&self, import: &str) -> Result<String> {
        Ok(import.to_string())
    }

    fn resolve_import(&self, _import: &str) -> Result<Option<String>> {
        Err(protospec_err!("null import resolver"))
    }

    fn resolve_ffi_transform(&self, transform: &str) -> Result<Option<ForeignTransformObj>> {
        Ok(match transform {
            "test_transform" => Some(Box::new(TestTransform)),
            _ => None,
        })
    }

    fn resolve_ffi_type(&self, type_: &str) -> Result<Option<ForeignTypeObj>> {
        Ok(match type_ {
            "test_type" => Some(Box::new(TestType)),
            _ => None,
        })
    }

    fn resolve_ffi_function(&self, name: &str) -> Result<Option<ForeignFunctionObj>> {
        Ok(None)
    }

    fn prelude_ffi_functions(&self) -> Result<HashMap<String, ForeignFunctionObj>> {
        Ok(Default::default())
    }
}

pub struct MockImportResolver(IndexMap<String, String>);

impl ImportResolver for MockImportResolver {
    fn normalize_import(&self, import: &str) -> Result<String> {
        Ok(import.to_string())
    }

    fn resolve_import(&self, import: &str) -> Result<Option<String>> {
        Ok(self.0.get(import).map(|x| x.clone()))
    }

    fn resolve_ffi_transform(&self, transform: &str) -> Result<Option<ForeignTransformObj>> {
        Ok(match transform {
            "test_transform" => Some(Box::new(TestTransform)),
            _ => None,
        })
    }

    fn resolve_ffi_type(&self, type_: &str) -> Result<Option<ForeignTypeObj>> {
        Ok(match type_ {
            "test_type" => Some(Box::new(TestType)),
            _ => None,
        })
    }

    fn resolve_ffi_function(&self, _name: &str) -> Result<Option<ForeignFunctionObj>> {
        Ok(None)
    }

    fn prelude_ffi_functions(&self) -> Result<HashMap<String, ForeignFunctionObj>> {
        Ok(Default::default())
    }
}
