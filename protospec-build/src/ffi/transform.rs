use proc_macro2::TokenStream;

use crate::FFIArgument;

pub type ForeignTransformObj = Box<dyn ForeignTransform + Send + Sync + 'static>;

pub trait ForeignTransform {
    fn decoding_gen(
        &self,
        input_stream: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    fn encoding_gen(
        &self,
        input_stream: TokenStream,
        arguments: Vec<TokenStream>,
        is_async: bool,
    ) -> TokenStream;

    fn arguments(&self) -> Vec<FFIArgument>;
}
