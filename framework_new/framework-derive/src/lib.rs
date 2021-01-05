use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Path, Type, parse_macro_input};

fn path_is_option(path: &Path) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.iter().next().unwrap().ident == "Option"
}

#[proc_macro_derive(FromArgs)]
pub fn from_args_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident;
    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("Only supported on structs");
    };
    let fields_decs = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let stmt = match ty {
            Type::Path(typepath) if path_is_option(&typepath.path) => {
                quote! {
                    let #name = match args.next() {
                        Some(s) => <#ty>::from_arg(s)?,
                        None => None
                    };
                }
            },
            _ => {
                quote! {
                    let #name = match args.next() {
                        Some(s) => <#ty>::from_arg(s)?,
                        None => return Err(ArgumentError::MissingArgument)
                    };
                }
            }
        };
        stmt
    });
    let field_names = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        name
    });

    let gen = quote! {
        impl FromArgs for #name {
            fn from_args(args: &mut Arguments) -> std::result::Result<Self, ArgumentError> {
                #(#fields_decs)*
                Ok(Self {
                    #(#field_names),*
                })
            }
        }
    };
    gen.into()
}
