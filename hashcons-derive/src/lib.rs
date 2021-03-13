use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, visit_mut::VisitMut, DeriveInput,
    GenericArgument, Ident, ItemImpl, PathArguments, Token, Type, Visibility::*,
};

#[proc_macro_derive(HashCons)]
pub fn hashcons_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let vis = if let Crate(_) = input.vis {
        quote! { (crate) }
    } else {
        quote! {}
    };
    let factory = format_ident!("{}_FACTORY", name.to_string().to_ascii_uppercase());
    let typename = format_ident!("hashcons_type_{}", name);
    let gen = quote! {
        #[allow(non_snake_case)]
        #[automatically_derived]
        pub #vis mod #typename {
            use std::convert::From;
            use hashconsing::{consign, HConsed, HashConsign};
            #[derive(PartialEq, Eq, Hash, Clone)]
            pub struct #name(pub(super) HConsed<super::#name>);

            impl From<#name> for super::#name {
                fn from(x: #name) -> Self {
                    x.0.get().clone()
                }
            }

            consign! {
                let #factory = consign(37) for super::#name;
            }

            impl From<super::#name> for #name {
                fn from(x: super::#name) -> Self {
                    #name(#factory.mk(x))
                }
            }
        }
    };
    gen.into()
}

struct HashConsTypeReplace {
    types: Vec<Ident>,
}

impl VisitMut for HashConsTypeReplace {
    fn visit_type_path_mut(&mut self, node: &mut syn::TypePath) {
        if self.types.iter().any(|x| node.path.is_ident(x)) {
            let p = node.path.clone();
            node.path = parse_quote!(crate::hashcons::#p);
        };
        syn::visit_mut::visit_type_path_mut(self, node)
    }
}

#[proc_macro_attribute]
pub fn hashcons(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parser = Punctuated::<Ident, Token![,]>::parse_terminated;
    let hashconsed = parse_macro_input!(attr with parser).into_iter();
    let input = parse_macro_input!(item as ItemImpl);
    let original = input.clone();
    let items = input.items;
    let self_ty = input.self_ty;
    let body = items
        .into_iter()
        .filter_map(|item| match item {
            syn::ImplItem::Method(m) if m.vis != syn::Visibility::Inherited => {
                let vis = m.vis;
                let mut sig = m.sig;
                let name = sig.ident.clone();
                let args = sig
                    .inputs
                    .clone()
                    .into_iter()
                    .map(|fnarg| match fnarg {
                        syn::FnArg::Receiver(receiver) => {
                            if receiver.reference.is_some() && receiver.mutability.is_none() {
                                quote!(#receiver.0.get())
                            } else {
                                quote!(#receiver.into())
                            }
                        }
                        syn::FnArg::Typed(pattype) => {
                            let pat = pattype.pat;
                            let ty = pattype.ty;
                            if hashconsed
                                .clone()
                                .find(|x| ty == parse_quote! { &#x })
                                .is_some()
                            {
                                quote!(#pat.0.get())
                            } else {
                                quote!(#pat.into())
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                (HashConsTypeReplace {
                    types: hashconsed.clone().collect(),
                })
                .visit_signature_mut(&mut sig);
                match sig.clone().output {
                    syn::ReturnType::Type(_, t)
                        if t == parse_quote! { Self }
                            || hashconsed
                                .clone()
                                .find(|x| t == parse_quote! { crate::hashcons::#x })
                                .is_some() =>
                    {
                        Some(quote! { #vis #sig {
                            <super::#self_ty>::#name(#(#args),*).into()
                        }})
                    }
                    syn::ReturnType::Type(_, t)
                        if is_mappable(&t).cloned() == Some(parse_quote!(Self))
                            || hashconsed
                                .clone()
                                .find(|x| {
                                    is_mappable(&t).cloned()
                                        == Some(parse_quote! { crate::hashcons::#x })
                                ||
                                    is_mappable(&t).cloned()
                                        == Some(parse_quote! { &crate::hashcons::#x })
                                })
                                .is_some() =>
                    {
                        Some(quote! { #vis #sig {
                            <super::#self_ty>::#name(#(#args),*).map(|x| x.into())
                        }})
                    }
                    syn::ReturnType::Type(_, t)
                        if is_collection(&t).cloned() == Some(parse_quote!(Self))
                            || hashconsed
                                .clone()
                                .find(|x| {
                                    is_collection(&t).cloned()
                                        == Some(parse_quote! { crate::hashcons::#x })
                                })
                                .is_some() =>
                    {
                        Some(quote! { #vis #sig {
                            <super::#self_ty>::#name(#(#args),*).into_iter().map(|x| x.into()).collect()
                        }})
                    }
                    _ => Some(quote! { #vis #sig {
                        <super::#self_ty>::#name(#(#args),*)
                    }}),
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let ty = self_ty.to_token_stream().to_string();
    let implname = format_ident!("hashcons_impl_{}", ty);
    let typename = format_ident!("hashcons_type_{}", ty);
    let gen = quote! {
        #original
        pub mod #implname {
            use super::*;
            impl super::#typename::#self_ty {
                #(#body)*
            }
        }
    };
    gen.into()
}

// Rust doesn't support a Functor typeclass
// Example: Given `Result<T, _>` outputs `Some(T)`
fn is_mappable(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(tpath) => {
            let t = tpath.path.segments.first()?;
            if !(t.ident == "Option" || t.ident == "Result") {
                return None;
            };
            if let PathArguments::AngleBracketed(genericargs) = &t.arguments {
                if let GenericArgument::Type(container) = genericargs.args.first()? {
                    return Some(container);
                }
            }
            None
        }
        _ => None,
    }
}

fn is_collection(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(tpath) => {
            let t = tpath.path.segments.first()?;
            if !(t.ident == "Vec") {
                return None;
            };
            if let PathArguments::AngleBracketed(genericargs) = &t.arguments {
                if let GenericArgument::Type(container) = genericargs.args.first()? {
                    return Some(container);
                }
            }
            None
        }
        _ => None,
    }
}
