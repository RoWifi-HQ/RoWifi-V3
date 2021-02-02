use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Lit,
    Meta, NestedMeta, Path, Type,
};

fn path_is_option(path: &Path) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.iter().next().unwrap().ident == "Option"
}

fn path_is_help(path: &Path) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments.iter().next().unwrap().ident == "help"
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
    let fields_decs = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let name = format!("{}", field_name);
        let struct_name = struct_name.clone();
        let ty = &f.ty;
        let stmt = match ty {
            Type::Path(typepath) if path_is_option(&typepath.path) => {
                quote! {
                    let #field_name = match args.next() {
                        Some(s) => <#ty>::from_arg(s)?,
                        None => None
                    };
                }
            }
            _ => {
                quote! {
                    let #field_name = match args.next() {
                        Some(s) => <#ty>::from_arg(s)?,
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
            Type::Path(typepath) if path_is_option(&typepath.path) => {
                quote! {
                    let #field_name = match options.get(&(#name)) {
                        Some(s) => <#ty>::from_interaction(s)?,
                        None => None
                    };
                }
            }
            _ => {
                quote! {
                    let #field_name = match options.get(&(#name)) {
                        Some(s) => <#ty>::from_interaction(s)?,
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
                match attr.parse_meta() {
                    Ok(Meta::List(mut nvs)) => match nvs.nested.pop().unwrap().into_value() {
                        NestedMeta::Meta(Meta::NameValue(nv)) if path_is_help(&nv.path) => {
                            match nv.lit {
                                Lit::Str(lit) => format!("{}: {}", name, lit.value()),
                                _ => panic!("This ident only accepts strings"),
                            }
                        }
                        _ => panic!("Not implemented for non-name val list"),
                    },
                    _ => panic!("Only meant for Meta"),
                }
            } else {
                format!("{}: No description", name)
            }
        })
        .join("\n");

    let usage = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            match ty {
                Type::Path(typepath) if path_is_option(&typepath.path) => {
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

            fn from_interaction(options: &[twilight_model::applications::command::CommandDataOption]) -> std::result::Result<Self, ArgumentError> {
                use twilight_model::applications::command::CommandDataOption;

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
