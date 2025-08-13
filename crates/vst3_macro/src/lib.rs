extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::ffi::c_char;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{FnArg, Ident, Meta, Pat, Token, TraitItemFn, Visibility, parse_macro_input};

struct Interface {
    _vis: Visibility,
    ident: Ident,
    _colon_token: Option<Token![:]>,
    parent: Option<Ident>,
    methods: Vec<TraitItemFn>,
}

impl Parse for Interface {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _vis = input.parse::<Visibility>()?;
        let _ = input.parse::<Token![trait]>()?;
        let ident = input.parse::<Ident>()?;

        let _colon_token: Option<Token![:]> = input.parse()?;

        let mut parent = None;
        if _colon_token.is_some() {
            parent = Some(input.parse::<Ident>()?);
        }

        let content: ParseBuffer;

        syn::braced!(content in input);
        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(content.parse::<TraitItemFn>()?);
        }

        Ok(Self {
            _vis,
            ident,
            _colon_token,
            parent,
            methods,
        })
    }
}

impl Interface {
    fn gen_tokens(&self, uid: [c_char; 16]) -> syn::Result<TokenStream2> {
        let ident = &self.ident;
        let vtable_name = quote::format_ident!("{}_Vtbl", ident);
        let com_trait = self.get_trait(uid);
        let vtable = self.gen_vtable(&vtable_name);

        Ok(quote! {
            #com_trait
            #vtable
        })
    }

    fn get_trait(&self, uid: [c_char; 16]) -> TokenStream2 {
        let ident = &self.ident;

        let vtable_name = quote::format_ident!("{}_Vtbl", ident);
        let impl_name = quote::format_ident!("{}_Impl", ident);

        let host_impl_name = quote::format_ident!("{}_HostImpl", ident);

        let impl_parent = &self.parent.clone().map_or(quote! {}, |p| {
            quote! {
                impl Marker<#p> for #ident {}
            }
        });

        let parent_supertrait = &self.parent.clone().map_or(quote! {}, |p| {
            if quote! {#p}.to_string() == "FUnknown" {
                quote! {
                    + FUnknown_Impl
                }
            } else {
                quote! {
                    + Marker<#p>
                }
            }
        });

        let methods = self
            .methods
            .iter()
            .filter(|method| method.default.is_none())
            .map(|method| {
                let method_ident = &method.sig.ident;
                let args = &method
                    .sig
                    .inputs
                    .iter()
                    .filter(|arg| matches!(arg, FnArg::Typed(..)))
                    .cloned()
                    .collect::<Punctuated<FnArg, Token![,]>>();

                let output = &method.sig.output;

                quote! {
                    #[inline]
                    unsafe fn #method_ident(&mut self, #args) #output;
                }
            })
            .collect::<Vec<_>>();

        let trait_methods = self.methods
            .iter()
            .filter(|method| !matches!(&method.attrs.get(0), Some(attr) if attr.path().is_ident("private")))
            .map(|method| {
                let method_ident = &method.sig.ident;
                let method_generics = &method.sig.generics;
                let args = &method.sig.inputs
                    .iter()
                    .filter(|arg| matches!(arg, FnArg::Typed(..)))
                    .cloned()
                    .collect::<Punctuated<FnArg, Token![,]>>();

                let arg_inputs = args.iter().map(|arg| {
                    match arg {
                        FnArg::Typed(pat) => {
                            *(pat.pat).clone()
                        }

                        _ => {
                            panic!("")
                        }
                    }
                }).collect::<Punctuated<Pat, Token![,]>>();

                let output = &method.sig.output;

                // let body = quote! {
                //     // (*(self as *mut _ as *mut #ident)).#method_impl_ident(#arg_inputs)
                //     (std::mem::transmute::<&'static Self::VTable, &#vtable_name>(self.vtable()).#method_ident)(self as *mut _ as *mut #ident, #arg_inputs)
                // };
                let body = method.default.clone().map_or(quote! {
                    (std::mem::transmute::<&'static Self::VTable, &#vtable_name>(self.vtable()).#method_ident)(self as *const _ as *mut #ident, #arg_inputs)
                }, |b| {
                    quote! {
                        #b
                    }
                });

                quote! {
                    #[inline]
                    unsafe fn #method_ident #method_generics(&self, #args) #output
                    where <Self as Interface>::VTable: 'static {
                        #body
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            #[repr(C)]
            #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
            pub struct #ident {
                pub vtable: &'static #vtable_name,
            }

            // impl #ident {
            //     #(#methods)*
            // }

            #[allow(non_camel_case_types)]
            pub trait #impl_name: Interface #parent_supertrait {
                #(#trait_methods)*
            }

            pub trait #host_impl_name {
                #(#methods)*
            }

            impl Marker<#ident> for #ident {}
            impl DefaultImplementation<FUnknown> for #ident {}
            #impl_parent

            unsafe impl Send for #ident {}
            unsafe impl Sync for #ident {}

            impl<T: Interface + Marker<#ident> #parent_supertrait> #impl_name for T {}

            impl Interface for #ident {
                type VTable = #vtable_name;
                const iid: FUID = [#(#uid),*];
                // const method_names: &'static [*const ()] = &[#(#method_names),*];

                fn vtable(&self) -> &'static Self::VTable {
                    self.vtable
                }
            }
        }
    }

    fn gen_vtable(&self, vtable_name: &Ident) -> TokenStream2 {
        let name = &self.ident;

        let methods = self
            .methods
            .iter()
            .filter(|method| method.default.is_none())
            .map(|method| {
                let ident = &method.sig.ident;
                let args = &method
                    .sig
                    .inputs
                    .iter()
                    .filter(|arg| matches!(arg, FnArg::Typed(..)))
                    .cloned()
                    .collect::<Punctuated<FnArg, Token![,]>>();
                let output = &method.sig.output;

                quote! {
                    pub #ident: unsafe extern "C" fn(this: *mut #name, #args) #output,
                }
            })
            .collect::<Vec<_>>();

        let parent = &self.parent.clone().map(|p| {
            let vtable_name = quote::format_ident!("{}_Vtbl", p);
            quote! {
                pub base: #vtable_name,
            }
        });

        quote! {
            #[allow(non_snake_case)]
            #[repr(C)]
            #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
            pub struct #vtable_name {
                #parent
                #(#methods)*
            }
        }
    }
}

fn inline_uid(l1: u32, l2: u32, l3: u32, l4: u32) -> [c_char; 16] {
    [
        (l1 & 0x000000FF) as c_char,
        ((l1 & 0x0000FF00) >> 8) as c_char,
        ((l1 & 0x00FF0000) >> 16) as c_char,
        ((l1 & 0xFF000000) >> 24) as c_char,
        ((l2 & 0x00FF0000) >> 16) as c_char,
        ((l2 & 0xFF000000) >> 24) as c_char,
        (l2 & 0x000000FF) as c_char,
        ((l2 & 0x0000FF00) >> 8) as c_char,
        ((l3 & 0xFF000000) >> 24) as c_char,
        ((l3 & 0x00FF0000) >> 16) as c_char,
        ((l3 & 0x0000FF00) >> 8) as c_char,
        (l3 & 0x000000FF) as c_char,
        ((l4 & 0xFF000000) >> 24) as c_char,
        ((l4 & 0x00FF0000) >> 16) as c_char,
        ((l4 & 0x0000FF00) >> 8) as c_char,
        (l4 & 0x000000FF) as c_char,
    ]

    // [
    //     ((l1 & 0xFF000000) >> 24) as c_char,
    //     ((l1 & 0x00FF0000) >> 16) as c_char,
    //     ((l1 & 0x0000FF00) >>  8) as c_char,
    //     ((l1 & 0x000000FF)      ) as c_char,
    //
    //     ((l2 & 0xFF000000) >> 24) as c_char,
    //     ((l2 & 0x00FF0000) >> 16) as c_char,
    //     ((l2 & 0x0000FF00) >>  8) as c_char,
    //     ((l2 & 0x000000FF)      ) as c_char,
    //
    //     ((l3 & 0xFF000000) >> 24) as c_char,
    //     ((l3 & 0x00FF0000) >> 16) as c_char,
    //     ((l3 & 0x0000FF00) >>  8) as c_char,
    //     ((l3 & 0x000000FF)      ) as c_char,
    //
    //     ((l4 & 0xFF000000) >> 24) as c_char,
    //     ((l4 & 0x00FF0000) >> 16) as c_char,
    //     ((l4 & 0x0000FF00) >>  8) as c_char,
    //     ((l4 & 0x000000FF)      ) as c_char,
    // ]
}

#[proc_macro_attribute]
pub fn interface(args: TokenStream, input: TokenStream) -> TokenStream {
    let clone = input.clone();
    let uid = parse_macro_input!(args with Punctuated::<syn::LitInt, Token![,]>::parse_terminated);
    let interface = parse_macro_input!(clone as Interface);

    let l1 = uid.get(0).unwrap().base10_parse().unwrap();
    let l2 = uid.get(1).unwrap().base10_parse().unwrap();
    let l3 = uid.get(2).unwrap().base10_parse().unwrap();
    let l4 = uid.get(3).unwrap().base10_parse().unwrap();

    let uid = inline_uid(l1, l2, l3, l4);

    let tokens = match interface.gen_tokens(uid) {
        Ok(t) => t,
        Err(e) => return e.to_compile_error().into(),
    };

    // println!("{}", tokens);

    tokens.into()
}

#[proc_macro_attribute]
pub fn implement(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute into a list of interfaces (traits) and the struct
    let interfaces = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input = parse_macro_input!(item as syn::ItemStruct);

    // Struct name
    let struct_name = &input.ident;

    // Collect vtable fields for each interface
    let mut vtable_fields = vec![];

    for interface in interfaces.iter() {
        if let Meta::Path(path) = interface {
            let trait_name = &path.segments.last().unwrap().ident;
            let vtable_field = format_ident!("{}_vtable", trait_name.to_string().to_lowercase());

            // Add a vtable field for each trait
            vtable_fields.push(quote! {
                pub #vtable_field: *const #trait_name
            });
        }
    }

    // Generate the final struct with vtable pointers
    let expanded = quote! {
        pub struct #struct_name {
            #(#vtable_fields),*
        }

        impl #struct_name {
            // Here you can add helper methods for setting up the vtable or other functionality
        }
    };

    println!("SKDOSSDOKSODKOSD: {:?}", expanded);

    TokenStream::from(expanded)
}
