use proc_macro::TokenStream;

use proc_macro2::Span;

use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, Lifetime, ReturnType, Signature, Token, Type};

use quote::quote;

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let fun = parse_macro_input!(input as ItemFn);

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = fun;

    let sig_span = sig.span();
    let Signature {
        asyncness,
        ident,
        mut inputs,
        output,
        ..
    } = sig;

    if asyncness.is_none() {
        return syn::Error::new(sig_span, "`async` keyword is missing")
            .to_compile_error()
            .into();
    }

    let output = match output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, t) => quote!(#t),
    };

    populate_lifetime(&mut inputs);

    let result = quote! {
        #(#attrs)*
        #vis fn #ident<'fut>(#inputs) -> futures::future::BoxFuture<'fut, #output> {
            use futures::future::FutureExt;

            async move {
                #block
            }.boxed()
        }
    };

    result.into()
}

fn populate_lifetime(inputs: &mut Punctuated<FnArg, Token![,]>) {
    for input in inputs {
        if let FnArg::Typed(kind) = input {
            if let Type::Reference(ty) = &mut *kind.ty {
                ty.lifetime = Some(Lifetime::new("'fut", Span::call_site()));
            }
        }
    }
}
