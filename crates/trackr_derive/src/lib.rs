use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type, Visibility};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(pkt), supports(struct_any))]
struct Tracked {
    ident: Ident,
    data: ast::Data<util::Ignored, TrackedField>,
    generics: syn::Generics,
    vis: syn::Visibility,
}

#[derive(Debug, FromField)]
#[darling(attributes(track))]
struct TrackedField {
    ident: Option<Ident>,
    vis: syn::Visibility,
    ty: Type,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    flag: bool,
    #[darling(default)]
    pub_: bool,
}

impl TrackedField {
    pub fn ident(&self) -> Ident {
        self.ident.clone().unwrap()
    }

    fn is_tracked(&self) -> bool {
        !self.flag && !self.skip
    }

    pub fn gen_impl(
        &self,
        vis: &Visibility,
        flag_ty: &Ident,
        tracker_field: &Ident,
    ) -> Option<TokenStream> {
        if self.skip {
            return None;
        }

        let vis = if self.pub_ {
            syn::Visibility::Public(syn::token::Pub::default())
        } else {
            vis.clone()
        };

        let ident = self.ident();
        let ty = &self.ty;

        let get = format_ident!("{ident}");
        let get_mut = format_ident!("{ident}_mut");

        Some(quote! {
            #vis fn #get(&self) -> &#ty {
                &self.#ident
            }

            #vis fn #get_mut(&mut self) -> trackr::TrackedFieldMut<'_, #ty, #flag_ty> {
                trackr::TrackedFieldMut::new(
                    #flag_ty::#ident,
                    &mut self.#ident,
                    &mut self.#tracker_field
                )
            }

        })
    }
}

/// Gets the type for the bitflags
fn bits_ty(n: usize) -> syn::Type {
    match n {
        n if n < 8 + 1 => syn::parse_quote!(u8),
        n if n < 16 + 1 => syn::parse_quote!(u16),
        n if n < 32 + 1 => syn::parse_quote!(u32),
        n if n < 64 + 1 => syn::parse_quote!(u64),
        n if n < 128 + 1 => syn::parse_quote!(u128),
        _ => panic!("Too many tracked fields, max is 128"),
    }
}

impl ToTokens for Tracked {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident.clone();
        let (imp, ty, wher) = self.generics.split_for_impl();
        let vis = self.vis.clone();

        let fields = self.data.as_ref().take_struct().expect("expected struct");

        // Find the field marked with tracker
        let tracker_field = fields
            .iter()
            .find(|field| field.flag)
            .expect("expected at least one #[track(field)] field");

        let tracker_field_id = tracker_field
            .ident
            .as_ref()
            .expect("Tracker field must be named");

        // Get all tracked fields
        let tracked_fields = fields.iter().filter(|field| field.is_tracked());
        // Get the type for the bitflags
        let bits_ty = bits_ty(tracked_fields.clone().count());

        // Auto generate all flags
        let flag_ty = format_ident!("{}Flags", ident);
        let flags = tracked_fields.clone().enumerate().map(|(i, field)| {
            let ident = field.ident.as_ref().unwrap();
            quote!( const #ident = 1 << #i; )
        });

        let field_impls = tracked_fields
            .clone()
            .map(|field| field.gen_impl(&field.vis, &flag_ty, tracker_field_id));

        quote!(
            trackr::bitflags! {
                #[repr(transparent)]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                #vis struct #flag_ty: #bits_ty {
                    #( #flags )*
                }
            }

            impl #imp #ident #ty #wher {
                #( #field_impls)*

                #vis fn flags(&self) -> #flag_ty {
                    self.#tracker_field_id
                }
            }
        )
        .to_tokens(tokens);
    }
}

#[proc_macro_derive(Tracked, attributes(track))]
pub fn tracked(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    let input = match Tracked::from_derive_input(&derive_input) {
        Ok(input) => input,
        Err(err) => return err.write_errors().into(),
    };

    input.to_token_stream().into()
}
