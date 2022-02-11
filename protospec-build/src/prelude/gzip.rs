use super::*;

pub struct GzipTransform;

impl ForeignTransform for GzipTransform {
    fn decoding_gen(
        &self,
        input_stream: TokenStream,
        _arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream {
        if is_async {
            quote! {
                async_compression::tokio::bufread::GzipDecoder::new(#input_stream)
            }
        } else {
            quote! {
                flate2::read::GzDecoder::new(#input_stream)
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
            quote! {
                async_compression::tokio::write::GzipEncoder::new(#input_stream)
            }
        } else {
            quote! {
                flate2::write::GzEncoder::new(#input_stream, flate2::Compression::default())
            }
        }
    }

    fn arguments(&self) -> Vec<FFIArgument> {
        vec![]
    }
}
