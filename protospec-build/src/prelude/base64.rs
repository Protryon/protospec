use super::*;

pub struct Base64Transform;

impl ForeignTransform for Base64Transform {
    fn decoding_sync_gen(&self, input_stream: TokenStream, arguments: Vec<TokenStream>) -> TokenStream {
        quote! {
            {
                base64::read::DecoderReader::new(#input_stream, base64::STANDARD)
            }
        }
    }

    fn arguments(&self) -> Vec<TransformArgument> {
        vec![]
    }

    fn encoding_sync_gen(&self, input_stream: TokenStream, arguments: Vec<TokenStream>) -> TokenStream {
        quote! {
            base64::write::EncoderWriter::new(#input_stream, base64::STANDARD)
        }
    }
}
