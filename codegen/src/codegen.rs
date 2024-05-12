use proc_macro2::TokenStream;
use quote::quote;

use crate::Model;

pub(crate) fn codegen(model: Model) -> TokenStream {
    let named_struct = model.build_struct();
    let params_struct = model.build_param_struct();
    let named_struct_impl = model.build_struct_impl();
    let named_struct_impl_task = model.build_struct_impl_for_task();

    quote! {
        #named_struct
        #params_struct

        #named_struct_impl
        #named_struct_impl_task
    }
}
