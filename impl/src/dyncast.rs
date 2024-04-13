use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, Error, GenericParam, Generics, ItemImpl, ItemTrait, Token,
    TraitItem, WherePredicate,
};

use crate::{args::Args, linker, parse::Item};

fn extract_generic_type_params_idents(generics: &Generics) -> TokenStream {
    let mut ts = TokenStream::new();
    for param in &generics.params {
        if let GenericParam::Type(ty) = param {
            ty.ident.to_tokens(&mut ts);
        }
        Token![,](Span::call_site()).to_tokens(&mut ts);
    }
    ts
}

pub fn expand_trait(item: &mut ItemTrait, _args: Args) -> Result<TokenStream, Error> {
    if let Some(first_const_param) = item.generics.const_params().next() {
        return Err(Error::new(
            first_const_param.span(),
            "const generics aren't allowed for dyncastable traits",
        ));
    }

    if let Some(first_lifetime_param) = item.generics.lifetimes().next() {
        return Err(Error::new(
            first_lifetime_param.span(),
            "lifetime generics aren't allowed for dyncastable traits",
        ));
    }

    let predicates = item
        .generics
        .type_params()
        .map(|type_param| {
            let ty = &type_param.ident;
            let predicate: WherePredicate = parse_quote! {
                #ty: 'static
            };
            predicate
        })
        .collect::<Vec<_>>();
    item.generics
        .make_where_clause()
        .predicates
        .extend(predicates);

    let trait_ident = &item.ident;
    let generics_lt = &item.generics.lt_token;
    let generics_params = &item.generics.params;
    let generics_gt = &item.generics.gt_token;
    let generics_where = &item.generics.where_clause;

    let generics_params_pass = extract_generic_type_params_idents(&item.generics);

    let trait_ident_with_params = quote!(
        #trait_ident #generics_lt #generics_params_pass #generics_gt
    );

    let dyncast_provider = format!("{}DyncastProvider", &item.ident);
    let dyncast_provider = Ident::new(
        &dyncast_provider,
        Span::call_site().located_at(item.ident.span()),
    );
    let dyncast_provider_with_params =
        quote!(#dyncast_provider #generics_lt #generics_params_pass #generics_gt);

    let dyncast_descriptor_ref = quote! {
        #[doc(hidden)]
        unsafe fn __dyncast_descriptor_ref() -> ::dyncast::private::Descriptor
        where
            Self: 'static + ::std::marker::Sized + #dyncast_provider_with_params
        {
            <Self as #dyncast_provider_with_params>::dyncast_descriptor()
        }
    };
    let dyncast_descriptor_ref = syn::parse2::<TraitItem>(dyncast_descriptor_ref).unwrap();

    item.items.push(dyncast_descriptor_ref);

    Ok(quote! {
        /// # Safety
        /// This trait must *not* be implemented on any type manually. Doing so might cause UB.
        #[doc(hidden)]
        unsafe trait #dyncast_provider #generics_lt #generics_params #generics_gt : #trait_ident_with_params
        #generics_where
        {
            #[inline(always)]
            fn dyncast_descriptor() -> ::dyncast::private::Descriptor
            where
                Self: 'static + ::std::marker::Sized,
            {
                unsafe {
                    ::dyncast::private::Descriptor::new(
                        ::std::any::TypeId::of::<Self>(),
                        ::std::any::TypeId::of::<dyn #trait_ident_with_params>(),
                        Self::dyncast_attach_vtable
                    )
                }
            }

            fn dyncast_attach_vtable(ptr: *const ()) -> *const dyn #trait_ident_with_params
            where
                Self: 'static + ::std::marker::Sized,
            {
                unsafe {
                    let vtable = ::dyncast::private::ptr::metadata(
                        ::std::ptr::null::<Self>() as *const dyn #trait_ident_with_params
                    );
                    ::dyncast::private::ptr::from_raw_parts(ptr, vtable)
                }
            }
        }

        unsafe impl<__T: #trait_ident_with_params, #generics_params>
            #dyncast_provider_with_params for __T
            #generics_where
        {}

        impl #generics_lt #generics_params #generics_gt ::dyncast::private::Dyncast
        for dyn #trait_ident_with_params
        #generics_where
        {
            fn dyncast_from<__T: ?::std::marker::Sized + ::std::any::Any>(
                __source: &__T
            ) -> ::std::option::Option<&Self> {
                use ::std::any::Any;

                let __map = unsafe {
                    ::dyncast::private::LazyTypeMap::<
                        dyn #trait_ident_with_params
                    >::current().get_or_init()
                };

                let __descriptor = __map.get(::std::any::Any::type_id(__source))?;

                Some(unsafe {
                    &*(__descriptor.attach_vtable_fn())(__source as *const _ as *const ())
                })
            }
        }
    })
}

pub fn expand_impl(item: &ItemImpl, _args: Args) -> Result<TokenStream, Error> {
    if let Some(span) = item
        .generics
        .const_params()
        .next()
        .map(|first_const_param| first_const_param.span())
        .or_else(|| {
            item.generics
                .type_params()
                .next()
                .map(|first_type_param| first_type_param.span())
        })
        .or_else(|| {
            item.generics
                .lifetimes()
                .next()
                .map(|first_lifetime_param| first_lifetime_param.span())
        })
    {
        return Err(Error::new(
            span.span(),
            "generics aren't allowed for dyncastable trait impls",
        ));
    }

    let trait_path = match &item.trait_ {
        Some((None, path, _)) => path,
        _ => {
            return Err(Error::new(
                item.impl_token.span,
                "inherent impls are invalid here",
            ))
        }
    };
    let self_ty = &*item.self_ty;

    let elf_section = linker::elf::SECTION;
    let macho_section = linker::macho::SECTION;
    let windows_section = linker::windows::SECTION;

    Ok(quote! {
        const _: () = {
            #[cfg_attr(
                any(target_os = "macos", target_os = "ios", target_os = "tvos"),
                link_section = #macho_section
            )]
            #[cfg_attr(
                any(target_os = "none", target_os = "linux", target_os = "freebsd"),
                link_section = #elf_section
            )]
            #[cfg_attr(
                target_os = "windows",
                link_section = #windows_section
            )]
            #[used]
            static REF_DYNCAST: ::dyncast::private::Entry = ::dyncast::private::Entry::new(
                <#self_ty as #trait_path>::__dyncast_descriptor_ref
            );
        };
    })
}

pub fn expand(item: &mut Item, args: Args) -> Result<TokenStream, Error> {
    match item {
        Item::Trait(item) => expand_trait(item, args),
        Item::Impl(item) => expand_impl(item, args),
    }
}
