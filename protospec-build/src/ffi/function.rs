use proc_macro2::TokenStream;

use crate::{asg::Type, FFIArgument, FFIArgumentValue};

pub type ForeignFunctionObj = Box<dyn ForeignFunction + Send + Sync + 'static>;

pub trait ForeignFunction {
    fn arguments(&self) -> Vec<FFIArgument>;

    fn return_type(&self) -> Type;

    fn call(&self, arguments: &[FFIArgumentValue]) -> TokenStream;
}
