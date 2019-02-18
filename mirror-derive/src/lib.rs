#![recursion_limit="128"]
#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::Span;
use std::collections::HashSet;
use syn::*;

struct Action {
    pub function:      LitStr,
    pub args:          usize,
}

fn impl_reflect_struct(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let mut field_id = Vec::<Member>::new();
    let mut field_ty = HashSet::new();

    match &ast.data {
        &Data::Struct(ref data) => {
            match &data.fields {
                &Fields::Named(ref fields) => {
                    for i in fields.named.iter() {
                        field_id.push(Member::Named(i.ident.clone().unwrap()));
                        field_ty.insert(i.ty.clone());
                    }
                },
                &Fields::Unnamed(ref fields) => {
                    for (i, f) in fields.unnamed.iter().enumerate() {
                        field_id.push(Member::Unnamed(Index::from(i)));
                        field_ty.insert(f.ty.clone());
                    }
                },
                &Fields::Unit => {
                    /* no fields */
                },
            }
        },
        _ => unreachable!(),
    };

    let mut impl_generics: Generics = ast.generics.clone();
    impl_generics.params.push(GenericParam::Lifetime(LifetimeDef::new(Lifetime::new("'de", Span::call_site()))));
    impl_generics.where_clause = Some(parse_quote!(where #(#field_ty: Reflect<'de>,)*));

    let (_, type_generics, _) = ast.generics.split_for_impl();
    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

    let field_str: Vec<Member> = field_id.clone();

    let tokens = quote! {
        impl #impl_generics Reflect<'de> for #name #type_generics #where_clause {
            fn command(&mut self, command: &Command) -> Result<(), Error> {
                use serde_json::from_value;
                match command {
                    &Command::Path { ref element, ref command } => {
                        #(if element == stringify!(#field_str) {
                            self.#field_id.command(command)?;
                            Ok(())
                        } else )* {
                            Err(Error::PathError)
                        }
                    },
                    &Command::Set { ref value } => {
                        *self = from_value(value.clone())?;
                        Ok(())
                    },
                    &Command::Call { ref key, ref arguments } => {
                        Ok(self._call(key.as_str(), arguments.as_slice())?)
                    },
                    &_ => {
                        Err(Error::IncompatibleCommand)
                    },
                }
            }
        }
    };

    tokens
}

fn impl_reflect_enum(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    unimplemented!();
}

fn attr_to_action(attr: &Meta) -> Action {
    if let &Meta::List(ref list) = attr {
        if list.ident == "Fn" {
            let mut function: Option<LitStr> = None;
            let mut args: usize = 0;
            for nest_meta in list.nested.iter() {
                if let &NestedMeta::Meta (ref meta) = nest_meta {
                    match meta {
                        &Meta::Word(_) => {
                            panic!("Invalid Fn attribute: Invalid value in list");
                        }
                        &Meta::NameValue(ref name_value) => {
                            match name_value.ident.to_string().as_ref() {
                                "name" => {
                                    if let &Lit::Str(ref lit) = &name_value.lit {
                                        function = Some(lit.clone())
                                    } else {
                                        panic!("Invalid Fn attribute: Expected a string for fn value");
                                    }
                                }
                                "args" => {
                                    if let &Lit::Str(ref lit) = &name_value.lit {
                                        args = lit.value().parse::<usize>().expect("Invalid Fn attribute: Expected a string that can parse into usize");
                                    } else {
                                        panic!("Invalid Fn attribute: Expected a string for args value");
                                    }
                                }
                                _ => {
                                    panic!("Invalid Fn attribute: Invalid value in list");
                                }
                            }
                        }
                        &Meta::List (_) => {
                            panic!("Invalid Fn attribute: Invalid value in list");
                        }
                    }
                } else {
                    panic!("Invalid Fn attribute: Invalid value in list");
                }
            }

            let function =
                function.expect("Invalid NodeAction attribute: Needs to specify a function");

            Action {
                function,
                args,
            }
        } else {
            panic!("Invalid Fn attribute: Needs to be a list")
        }
    } else {
        panic!("Invalid Fn attribute: Needs to be a list")
    }
}

fn impl_reflect_actions(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let impl_generics: Generics = ast.generics.clone();

    let (_, type_generics, _) = ast.generics.split_for_impl();
    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

    let mut actions = Vec::new();

    for attr in ast.attrs.iter() {
        if let Some(Meta::List(list)) = attr.interpret_meta() {
            if list.ident == "ReflectFn" {
                for nest_meta in list.nested.iter() {
                    if let &NestedMeta::Meta (ref sub_attr) = nest_meta {
                        actions.push(attr_to_action(sub_attr));
                    } else {
                        panic!("Invalid ReflectFn attribute: Needs to be a list of Fn")
                    }
                }
            }
        }
    }

    let mut arms: Vec<proc_macro2::TokenStream> = vec!();

    for action in actions {
        let action_name = &action.function;
        let span = action.function.span();
        let function_name = Ident::new(action.function.value().as_ref(), span);

        let mut args: Vec<proc_macro2::TokenStream> = vec!();
        for i in 0..action.args {
            args.push(quote_spanned! { Span::call_site() =>
                serde_json::from_value(arguments[#i].clone())?
            });
        }

        let function_call = quote_spanned!{ span =>
            #function_name(#( #args ),*)
        };

        arms.push(quote_spanned!{ Span::call_site() =>
            #action_name => {
                self.#function_call;
                Ok(())
            }
        });
    }

    let match_statement = quote_spanned!{ Span::call_site() =>
        match key {
            #( #arms )*
            _ => Err(Error::InvalidCommand)
        }
    };

    let tokens = quote! {
        impl #impl_generics #name #type_generics #where_clause {
            fn _call(&mut self, key: &str, arguments: &[serde_json::Value]) -> Result<(), Error> {
                #match_statement
            }
        }
    };

    tokens
}

fn reflect(input: &DeriveInput) -> TokenStream {
    let reflect_impl = match &input.data {
        &Data::Struct(_) => {
            impl_reflect_struct(input)
        },
        &Data::Enum(_) => {
            impl_reflect_enum(input)
        },
        &Data::Union(_) => {
            panic!("union not supported")
        }
    };

    let actions_impl = impl_reflect_actions(input);

    let tokens = quote! {
        #reflect_impl
        #actions_impl
    };

    tokens.into()
}

#[proc_macro_derive(Reflect, attributes(ReflectFn))]
pub fn derive_reflect(input: TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    reflect(&input)
}