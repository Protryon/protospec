use super::*;

pub struct Base64Transform;

impl ForeignTransform for Base64Transform {
    fn decoding_gen(
        &self,
        input_stream: TokenStream,
        _arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        if is_async {
            panic!("base64 async not implemented");
        }
        quote! {
            {
                base64::read::DecoderReader::new(#input_stream, base64::STANDARD)
            }
        }
    }

    fn encoding_gen(
        &self,
        input_stream: TokenStream,
        _arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        if is_async {
            panic!("base64 async not implemented");
        }
        quote! {
            base64::write::EncoderWriter::new(#input_stream, base64::STANDARD)
        }
    }

    fn arguments(&self) -> Vec<FFIArgument> {
        vec![]
    }
}
