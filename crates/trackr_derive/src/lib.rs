use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type, Visibility};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
struct Tracked {
    ident: Ident,
    data: ast::Data<util::Ignored, TrackedField>,
    generics: syn::Generics,
    vis: Visibility,
}

#[derive(Debug, FromField)]
#[darling(attributes(track))]
struct TrackedField {
    ident: Option<Ident>,
    vis: Visibility,
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
        self.ident.clone().expect("expected named field")
    }

    fn is_tracked(&self) -> bool {
        !self.flag && !self.skip
    }

    pub fn gen_impl(
        &self,
        vis: &Visibility,
        flag_ty: &Ident,
        flag_field: &Ident,
    ) -> Option<TokenStream> {
        if self.skip {
            return None;
        }

        let vis = if self.pub_ {
            Visibility::Public(syn::token::Pub::default())
        } else {
            vis.clone()
        };

        let ident = self.ident();
        let ty = &self.ty;
        let get: Ident = format_ident!("{ident}");
        let get_mut = format_ident!("{ident}_mut");

        Some(quote! {
            #vis fn #get(&self) -> &#ty {
                &self.#ident
            }

            #vis fn #get_mut(&mut self) -> trackr::TrackedField<'_, #ty, #flag_ty> {
                trackr::TrackedField::new(
                    #flag_ty::#ident,
                    &mut self.#flag_field,
                    &mut self.#ident
                )
            }

        })
    }
}

/// Gets the type for the bitflags
fn bits_ty(n: usize) -> Option<syn::Type> {
    match n {
        n if n < 8 + 1 => Some(syn::parse_quote!(u8)),
        n if n < 16 + 1 => Some(syn::parse_quote!(u16)),
        n if n < 32 + 1 => Some(syn::parse_quote!(u32)),
        n if n < 64 + 1 => Some(syn::parse_quote!(u64)),
        n if n < 128 + 1 => Some(syn::parse_quote!(u128)),
        _ => None,
    }
}

struct TrackedOutput<'a> {
    ident: &'a Ident,
    vis: &'a Visibility,
    flag_field: &'a TrackedField,
    tracked_fields: Vec<&'a &'a TrackedField>,
    bit_ty: Type,
    generics: &'a syn::Generics,
}

impl ToTokens for TrackedOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let (imp, ty, wher) = self.generics.split_for_impl();
        let vis = self.vis;

        // The field that holds the flags
        let flag_field = self
            .flag_field
            .ident
            .as_ref()
            .expect("Tracker field must be named");

        // Auto generate all flags
        let flag_ty = format_ident!("{ident}Flags");
        let flags = self.tracked_fields.iter().enumerate().map(|(i, field)| {
            let ident = field.ident.as_ref().expect("expected named field");
            quote!( const #ident = 1 << #i; )
        });

        // Generate impls for each tracked field
        let field_impls = self
            .tracked_fields
            .iter()
            .map(|field| field.gen_impl(&field.vis, &flag_ty, flag_field));

        let bits_ty = &self.bit_ty;

        quote!(
            trackr::__reexport::bitflags! {
                #[repr(transparent)]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                #vis struct #flag_ty: #bits_ty {
                    #( #flags )*
                }
            }

            impl #imp #ident #ty #wher {
                #( #field_impls)*
            }

            impl trackr::TrackedStruct for #ident #ty #wher {
                type Flags = #flag_ty;
                fn flags(&self) -> Self::Flags {
                    self.#flag_field
                }

                fn flags_mut(&mut self) -> &mut Self::Flags {
                    &mut self.#flag_field
                }
            }
        )
        .to_tokens(tokens);
    }
}

fn tracked_impl(input: Tracked) -> syn::Result<proc_macro2::TokenStream> {
    // Validate before generating tokens so we can emit nice span errors instead of panicking.
    let fields = input.data.as_ref().take_struct().ok_or_else(|| {
        syn::Error::new_spanned(&input.ident, "#[derive(Tracked)] only supports structs")
    })?;

    // Collect all flag fields (#[track(flag)])
    let flag_fields: Vec<_> = fields.iter().filter(|f| f.flag).collect();
    if flag_fields.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "missing a field marked with #[track(flag)]",
        ));
    }
    if flag_fields.len() > 1 {
        // Combine errors for each additional flag field to give user precise spans.
        let mut err = syn::Error::new_spanned(
            flag_fields[0]
                .ident
                .as_ref()
                .expect("darling guarantees named field"),
            "multiple #[track(flag)] fields found (first)",
        );
        for extra in &flag_fields[1..] {
            if let Some(id) = &extra.ident {
                err.combine(syn::Error::new_spanned(
                    id,
                    "additional #[track(flag)] field here",
                ));
            }
        }
        return Err(err);
    }

    // Safe to unwrap since we checked len() above
    let flag_field = flag_fields[0];

    // Obtain tracked fields
    let tracked_fields: Vec<_> = fields.iter().filter(|f| f.is_tracked()).collect();
    let tracked_count = tracked_fields.len();

    // Determine the bit type we need to store the flags
    let bit_ty = bits_ty(tracked_count).ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            format!("too many tracked fields: {tracked_count} (maximum supported is 128)"),
        )
    })?;

    Ok(TrackedOutput {
        ident: &input.ident,
        generics: &input.generics,
        vis: &input.vis,
        flag_field,
        tracked_fields,
        bit_ty,
    }
    .to_token_stream())
}

#[proc_macro_derive(Tracked, attributes(track))]
pub fn tracked(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = syn::parse_macro_input!(item as syn::DeriveInput);

    let input = match Tracked::from_derive_input(&derive_input) {
        Ok(input) => input,
        Err(err) => return err.write_errors().into(),
    };
    tracked_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
