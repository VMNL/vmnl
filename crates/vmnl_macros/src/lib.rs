// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Proc macros for VMNL public derives.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Error as SynError, Fields, Ident, LitStr,
    Result as SynResult, Token,
};

/// Derives Vulkano `Vertex` through VMNL's raw reexport.
#[proc_macro_derive(Vertex, attributes(name, format))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    derive_vertex_impl(&input)
        .unwrap_or_else(SynError::into_compile_error)
        .into()
}

/// Derives bytemuck `Pod` through VMNL's raw reexport.
#[proc_macro_derive(Pod)]
pub fn derive_pod(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    derive_marker_impl(&input, MarkerTrait::Pod)
        .unwrap_or_else(SynError::into_compile_error)
        .into()
}

/// Derives bytemuck `Zeroable` through VMNL's raw reexport.
#[proc_macro_derive(Zeroable)]
pub fn derive_zeroable(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    derive_marker_impl(&input, MarkerTrait::Zeroable)
        .unwrap_or_else(SynError::into_compile_error)
        .into()
}

fn derive_marker_impl(input: &DeriveInput, marker: MarkerTrait) -> SynResult<TokenStream2> {
    let vmnl: TokenStream2 = vmnl_crate_path()?;
    let bytemuck = quote!(#vmnl::raw::__private::bytemuck);
    let ident: &Ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let trait_name = marker.ident();
    let assertions = marker.assertions(input, &bytemuck)?;

    Ok(quote! {
        #assertions
        unsafe impl #impl_generics #bytemuck::#trait_name for #ident #ty_generics #where_clause {}
    })
}

fn derive_vertex_impl(input: &DeriveInput) -> SynResult<TokenStream2> {
    let vmnl = vmnl_crate_path()?;
    let vulkano = quote!(#vmnl::raw::__private::vulkano);
    let struct_name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let (impl_generics, ty_generics_impl, where_clause) = input.generics.split_for_impl();
    let members = vertex_members(named_vertex_fields(input)?, &vulkano)?;

    let function_body = quote! {
        #members

        #vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
            members,
            stride: ::std::mem::size_of::<#struct_name #ty_generics>() as u32,
            input_rate: #vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex,
        }
    };

    Ok(quote! {
        unsafe impl #impl_generics #vulkano::pipeline::graphics::vertex_input::Vertex
            for #struct_name #ty_generics_impl #where_clause
        {
            #[inline(always)]
            fn per_vertex() -> #vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                #function_body
            }

            #[inline(always)]
            fn per_instance() -> #vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                Self::per_vertex().per_instance()
            }

            #[inline(always)]
            fn per_instance_with_divisor(
                divisor: u32,
            ) -> #vulkano::pipeline::graphics::vertex_input::VertexBufferDescription {
                Self::per_vertex().per_instance_with_divisor(divisor)
            }
        }
    })
}

fn named_vertex_fields(input: &DeriveInput) -> SynResult<&Punctuated<syn::Field, Token![,]>> {
    let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = &input.data
    else {
        return Err(SynError::new_spanned(
            input,
            "raw::Vertex expects a struct with named fields",
        ));
    };

    Ok(&fields.named)
}

fn vertex_members(
    fields: &Punctuated<syn::Field, Token![,]>,
    vulkano: &TokenStream2,
) -> SynResult<TokenStream2> {
    let mut members = quote! {
        let mut offset = 0usize;
        let mut members = ::std::collections::HashMap::default();
    };

    for field in fields {
        let Some(field_ident) = field.ident.as_ref() else {
            return Err(SynError::new_spanned(
                field,
                "raw::Vertex expects named fields",
            ));
        };
        let field_name_lit = LitStr::new(&field_ident.to_string(), Span::call_site());
        let field_ty = &field.ty;
        let names = vertex_field_names(field, field_name_lit.clone())?;
        let format = vertex_field_format(field, vulkano)?;

        for name in &names {
            members = quote! {
                #members

                {
                    let field_align = ::std::mem::align_of::<#field_ty>();
                    offset = (offset + field_align - 1) & !(field_align - 1);

                    let field_size = ::std::mem::size_of::<#field_ty>();
                    let format = #format;
                    let format_size = format.block_size() as usize;
                    let num_elements = field_size / format_size;
                    let remainder = field_size % format_size;
                    ::std::assert!(
                        remainder == 0,
                        "struct field `{}` size does not fit multiple of format size",
                        #field_name_lit,
                    );

                    members.insert(
                        #name.to_string(),
                        #vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                            offset: offset as u32,
                            format,
                            num_elements: num_elements as u32,
                            stride: format_size as u32,
                        },
                    );

                    offset += field_size;
                }
            };
        }
    }

    Ok(members)
}

fn vertex_field_names(field: &syn::Field, fallback: LitStr) -> SynResult<Vec<LitStr>> {
    for attr in &field.attrs {
        if attr.path().is_ident("name") {
            return attr
                .parse_args_with(NameMeta::parse)
                .map(|meta| meta.lit_str_list.into_iter().collect());
        }
    }

    Ok(vec![fallback])
}

fn vertex_field_format(field: &syn::Field, vulkano: &TokenStream2) -> SynResult<TokenStream2> {
    for attr in &field.attrs {
        if attr.path().is_ident("format") {
            let format_ident = attr.parse_args_with(Ident::parse)?;
            return Ok(quote! {
                #vulkano::format::Format::#format_ident
            });
        }
    }

    Err(SynError::new_spanned(
        field,
        "expected #[format(...)] with a valid vulkano format",
    ))
}

fn vmnl_crate_path() -> SynResult<TokenStream2> {
    match crate_name("vmnl") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(_) => match crate_name("vmnl-graphics") {
            Ok(FoundCrate::Itself) => Ok(quote!(crate)),
            Ok(FoundCrate::Name(name)) => {
                let ident = Ident::new(&name, Span::call_site());
                Ok(quote!(::#ident))
            }
            Err(err) => Err(SynError::new(Span::call_site(), err)),
        },
    }
}

struct NameMeta {
    lit_str_list: Punctuated<LitStr, Token![,]>,
}

impl Parse for NameMeta {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        Ok(Self {
            lit_str_list: input.parse_terminated(ParseBuffer::parse, Token![,])?,
        })
    }
}

#[derive(Clone, Copy)]
enum MarkerTrait {
    Pod,
    Zeroable,
}

impl MarkerTrait {
    fn ident(self) -> TokenStream2 {
        match self {
            Self::Pod => quote!(Pod),
            Self::Zeroable => quote!(Zeroable),
        }
    }

    fn assertions(self, input: &DeriveInput, bytemuck: &TokenStream2) -> SynResult<TokenStream2> {
        let field_types = field_types(input)?;
        let trait_name = self.ident();
        let field_assertions = field_types.iter().map(|field_ty| {
            quote! {
                let _ = AssertField::<#field_ty>(::core::marker::PhantomData);
            }
        });

        if matches!(self, Self::Zeroable) {
            return Ok(quote! {
                const _: () = {
                    struct AssertField<T: #bytemuck::#trait_name>(::core::marker::PhantomData<T>);
                    #(#field_assertions)*
                };
            });
        }

        if !has_stable_repr(input) {
            return Err(SynError::new_spanned(
                input,
                "raw::Pod requires #[repr(C)] or #[repr(transparent)]",
            ));
        }

        let padding_assertion = if input.generics.params.is_empty() {
            padding_assertion(input, &field_types)
        } else {
            TokenStream2::new()
        };

        Ok(quote! {
            const _: () = {
                struct AssertField<T: #bytemuck::#trait_name>(::core::marker::PhantomData<T>);
                #(#field_assertions)*
                #padding_assertion
            };
        })
    }
}

fn field_types(input: &DeriveInput) -> SynResult<Vec<&syn::Type>> {
    let Data::Struct(DataStruct { fields, .. }) = &input.data else {
        return Err(SynError::new_spanned(
            input,
            "raw marker derives expect a struct",
        ));
    };

    Ok(fields.iter().map(|field| &field.ty).collect())
}

fn has_stable_repr(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|attr| {
        attr.path().is_ident("repr")
            && attr
                .parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)
                .is_ok_and(|items| {
                    items
                        .iter()
                        .any(|item| item == "C" || item == "transparent")
                })
    })
}

fn padding_assertion(input: &DeriveInput, field_types: &[&syn::Type]) -> TokenStream2 {
    let ident = &input.ident;
    let mut layout_steps = TokenStream2::new();

    for field_ty in field_types {
        layout_steps = quote! {
            #layout_steps
            let field_align = ::core::mem::align_of::<#field_ty>();
            offset = (offset + field_align - 1) & !(field_align - 1);
            offset += ::core::mem::size_of::<#field_ty>();
        };
    }

    quote! {
        let mut offset = 0usize;
        #layout_steps
        assert!(
            offset == ::core::mem::size_of::<#ident>(),
            "raw::Pod rejects structs with padding bytes",
        );
    }
}
