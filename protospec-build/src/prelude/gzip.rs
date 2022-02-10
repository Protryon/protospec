use super::*;

pub struct GzipTransform;

impl ForeignTransform for GzipTransform {
    fn decoding_sync_gen(&self, input_stream: TokenStream, arguments: Vec<TokenStream>) -> TokenStream {
        quote! {
            flate2::read::GzDecoder::new(#input_stream)
        }
    }

    fn encoding_sync_gen(&self, input_stream: TokenStream, arguments: Vec<TokenStream>) -> TokenStream {
        quote! {
            flate2::write::GzEncoder::new(#input_stream, flate2::Compression::default())
        }
    }

    fn arguments(&self) -> Vec<TransformArgument> {
        vec![]
    }
}
