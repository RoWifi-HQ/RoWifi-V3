use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Lit,
    Meta, NestedMeta, Path, Type,
};

fn path_is(path: &Path, slug: &str) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.iter().next().unwrap().ident.eq(slug)
}

fn builder(field: &Field) -> Option<&Attribute> {
    for attr in &field.attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "arg" {
            return Some(attr);
        }
    }
    None
}

#[proc_macro_derive(FromArgs, attributes(arg))]
pub fn from_args_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = ast.ident;
    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("Only supported on structs");
    };
    let fields_decs = fields.iter().enumerate().map(|(i, f)| {
        let field_name = f.ident.as_ref().unwrap();
        let name = format!("{}", field_name);
        let struct_name = struct_name.clone();
        let ty = &f.ty;

        if let Some(attr) = builder(f) {
            if let Ok(Meta::List(nvs)) = attr.parse_meta() {
                for nv in nvs.nested {
                    if let NestedMeta::Meta(Meta::Path(path)) = nv {
                        if path_is(&path, "rest") {
                            if i != fields.len() - 1 {
                                panic!("This attribute may only be used on the last field of a struct")
                            }
                            return quote! {
                                let #field_name = match args.rest().map(|s| <#ty>::from_arg(s.as_str())) {
                                    Some(Ok(s)) => s,
                                    Some(Err(err)) => return Err(ArgumentError::ParseError{
                                        expected: err.0,
                                        usage: <#struct_name>::generate_help(),
                                        name: #name
                                    }),
                                    None => return Err(ArgumentError::MissingArgument {
                                        usage: <#struct_name>::generate_help(),
                                        name: #name
                                    })
                                };
                            }
                        }
                    }
                }
            }
        }

        let stmt = match ty {
            Type::Path(typepath) if path_is(&typepath.path, "Option") => {
                quote! {
                    let #field_name = match args.next().map(<#ty>::from_arg) {
                        Some(Ok(s)) => s,
                        Some(Err(err)) => return Err(ArgumentError::ParseError{
                            expected: err.0,
                            usage: <#struct_name>::generate_help(),
                            name: #name
                        }),
                        None => None
                    };
                }
            }
            _ => {
                quote! {
                    let #field_name = match args.next().map(<#ty>::from_arg) {
                        Some(Ok(s)) => s,
                        Some(Err(err)) => return Err(ArgumentError::ParseError{
                            expected: err.0,
                            usage: <#struct_name>::generate_help(),
                            name: #name
                        }),
                        None => return Err(ArgumentError::MissingArgument {
                            usage: <#struct_name>::generate_help(),
                            name: #name
                        })
                    };
                }
            }
        };
        stmt
    });

    let field_interaction_decs = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let name = format!("{}", field_name);
        let struct_name = struct_name.clone();
        let ty = &f.ty;
        let stmt = match ty {
            Type::Path(typepath) if path_is(&typepath.path, "Option") => {
                quote! {
                    let #field_name = match options.get(&(#name)).map(|s| <#ty>::from_interaction(*s)) {
                        Some(Ok(s)) => s,
                        Some(Err(err)) => return Err(ArgumentError::ParseError{
                            expected: err.0,
                            usage: <#struct_name>::generate_help(),
                            name: #name
                        }),
                        None => None
                    };
                }
            }
            _ => {
                quote! {
                    let #field_name = match options.get(&(#name)).map(|s| <#ty>::from_interaction(*s)) {
                        Some(Ok(s)) => s,
                        Some(Err(err)) => return Err(ArgumentError::ParseError{
                            expected: err.0,
                            usage: <#struct_name>::generate_help(),
                            name: #name
                        }),
                        None => return Err(ArgumentError::MissingArgument {
                            usage: <#struct_name>::generate_help(),
                            name: #name
                        })
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
    let field_interaction_names = field_names.clone();

    let fields_help = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            if let Some(attr) = builder(f) {
                if let Ok(Meta::List(nvs)) = attr.parse_meta() {
                    for nv in nvs.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = nv {
                            if path_is(&nv.path, "help") {
                                if let Lit::Str(lit) = nv.lit {
                                    return format!("{}: {}", name, lit.value());
                                }
                            }
                        }
                    }
                }
            }
            format!("{}: No description", name)
        })
        .join("\n");

    let usage = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            match ty {
                Type::Path(typepath) if path_is(&typepath.path, "Option") => {
                    format!("[{}] ", field_name)
                }
                _ => {
                    format!("<{}> ", field_name)
                }
            }
        })
        .join(" ");

    let gen = quote! {
        impl FromArgs for #struct_name {
            fn from_args(args: &mut Arguments) -> std::result::Result<Self, ArgumentError> {
                #(#fields_decs)*
                Ok(Self {
                    #(#field_names),*
                })
            }

            fn from_interaction(options: &[twilight_model::applications::interaction::CommandDataOption]) -> std::result::Result<Self, ArgumentError> {
                use twilight_model::applications::interaction::CommandDataOption;

                let options = options.iter().map(|c| {
                    match c {
                        CommandDataOption::Boolean {name, ..}
                        | CommandDataOption::Integer {name, ..}
                        | CommandDataOption::String {name, ..}
                        | CommandDataOption::SubCommand {name, ..}
                            => (name.as_str(), c),
                    }
                }).collect::<std::collections::HashMap<&str, &CommandDataOption>>();

                #(#field_interaction_decs)*
                Ok(Self {
                    #(#field_interaction_names),*
                })
            }

            fn generate_help() -> (&'static str, &'static str) {
                (#usage, #fields_help)
            }
        }
    };
    gen.into()
}
