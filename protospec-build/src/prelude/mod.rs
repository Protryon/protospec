use crate::asg::*;
use crate::ast::ScalarType;
use crate::import::*;
use crate::result::*;
use proc_macro2::TokenStream;
use quote::*;

mod resolver;
pub use resolver::*;

// requires base64 crate
mod base64;
pub use base64::*;

// requires flate2 crate
mod gzip;
pub use gzip::*;

mod var;
pub use var::*;

mod utf8;
pub use utf8::*;

mod len;
pub use len::*;

pub fn map_async(is_async: bool) -> TokenStream {
    if is_async {
        quote! { .await }
    } else {
        quote! {}
    }
}
