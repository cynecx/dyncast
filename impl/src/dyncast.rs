use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, Error, GenericParam, Generics, ItemImpl, ItemTrait, Token,
    TraitItem, WherePredicate,
};

use crate::{args::Args, linker, parse::Item};

enum AsmSectionKind {
    DyncastDescriptorSymbol,
    ZeroData,
}

fn generate_asm_sections(
    args: &Args,
    trait_ident: &Ident,
    kind: AsmSectionKind,
    is_global: bool,
) -> TokenStream {
    let elf_section = linker::elf::section(trait_ident, args.seed);
    let macho_section = linker::macho::section(trait_ident, args.seed);

    let push_elf_section = format!(".pushsection {elf_section}");
    let push_macho_section = format!(".pushsection {macho_section}");

    let (data_32, data_64, bindings, options) = match kind {
        AsmSectionKind::DyncastDescriptorSymbol => (
            ".long {data}",
            ".quad {data}",
            quote!(data = sym Self::dyncast_descriptor,),
            quote!(nomem, preserves_flags, nostack),
        ),
        AsmSectionKind::ZeroData => (".long 0", ".quad 0", quote!(), quote!()),
    };

    let asm_kind = if is_global {
        quote!(::std::arch::global_asm!)
    } else {
        quote!(::std::arch::asm!)
    };

    quote! {
        #[cfg(all(any(target_os = "macos", target_os = "ios", target_os = "tvos"), target_pointer_width = "64"))]
        #asm_kind(
            #push_macho_section,
            ".p2align 3, 0",
            #data_64,
            ".popsection",
            #bindings
            options(#options)
        );
        #[cfg(all(any(target_os = "macos", target_os = "ios", target_os = "tvos"), target_pointer_width = "32"))]
        #asm_kind(
            #push_macho_section,
            ".p2align 2, 0",
            #data_32,
            ".popsection",
            #bindings
            options(#options)
        );
        #[cfg(all(any(target_os = "none", target_os = "linux", target_os = "freebsd"), target_pointer_width = "64"))]
        #asm_kind(
            #push_elf_section,
            ".p2align 3, 0",
            #data_64,
            ".popsection",
            #bindings
            options(#options)
        );
        #[cfg(all(any(target_os = "none", target_os = "linux", target_os = "freebsd"), target_pointer_width = "32"))]
        #asm_kind(
            #push_elf_section,
            ".p2align 2, 0",
            #data_32,
            ".popsection",
            #bindings
            options(#options)
        );
    }
}

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

pub fn expand_trait(item: &mut ItemTrait, args: Args) -> Result<TokenStream, Error> {
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

    let has_generics = !item.generics.params.is_empty();

    let inner_trait_ident = Ident::new(&format!("Inner{}", trait_ident), Span::call_site());

    let dyncast_provider = format!("{}DyncastProvider", &item.ident);
    let dyncast_provider = Ident::new(
        &dyncast_provider,
        Span::call_site().located_at(item.ident.span()),
    );
    let dyncast_provider_with_params =
        quote!(#dyncast_provider #generics_lt #generics_params_pass #generics_gt);

    let descriptor_entry = generate_asm_sections(
        &args,
        trait_ident,
        AsmSectionKind::DyncastDescriptorSymbol,
        false,
    );
    let empty_descriptor_entry =
        generate_asm_sections(&args, trait_ident, AsmSectionKind::ZeroData, true);

    let elf_section_start = linker::elf::section_start(trait_ident, args.seed);
    let elf_section_stop = linker::elf::section_stop(trait_ident, args.seed);

    let macho_section_start = linker::macho::section_start(trait_ident, args.seed);
    let macho_section_stop = linker::macho::section_stop(trait_ident, args.seed);

    let generics_type_id = if has_generics {
        quote! {
            let generics_type_id = Some(
                ::dyncast::private::TypeId::of::<(#generics_params_pass)>()
            );
        }
    } else {
        quote! {
            let generics_type_id = None;
        }
    };

    let make_dyncast_descriptor = if has_generics {
        quote! {
            ::dyncast::private::Descriptor::new_generics(
                ::std::any::TypeId::of::<Self>(),
                ::std::any::TypeId::of::<(#generics_params_pass)>(),
                Self::dyncast_attach_vtable
            )
        }
    } else {
        quote! {
            ::dyncast::private::Descriptor::new(
                ::std::any::TypeId::of::<Self>(),
                Self::dyncast_attach_vtable
            )
        }
    };

    let make_map = if has_generics {
        quote! {
            trait #inner_trait_ident: 'static {}
            let lazy = ::dyncast::private::LazyTypeMap::<dyn #inner_trait_ident>::current();
            let map = unsafe { lazy.get_or_init(&DYNCAST_START, &DYNCAST_STOP) };
        }
    } else {
        quote! {
            trait #inner_trait_ident: 'static {}
            static LAZY_MAP: ::dyncast::private::LazyTypeMap::<dyn #inner_trait_ident> =
                ::dyncast::private::LazyTypeMap::new();
            let map = unsafe { LAZY_MAP.get_or_init(&DYNCAST_START, &DYNCAST_STOP) };
        }
    };

    let dyncast_descriptor_ref = quote! {
        unsafe fn __dyncast_descriptor_ref()
        where
            Self: 'static + ::std::marker::Sized + #dyncast_provider_with_params
        {
            let _ = <Self as #dyncast_provider_with_params>::dyncast_descriptor();
        }
    };
    let dyncast_descriptor_ref = syn::parse2::<TraitItem>(dyncast_descriptor_ref).unwrap();

    item.items.push(dyncast_descriptor_ref);

    Ok(quote! {
        unsafe trait #dyncast_provider #generics_lt #generics_params #generics_gt : #trait_ident_with_params
        #generics_where
        {
            fn dyncast_descriptor() -> ::dyncast::private::Descriptor
            where
                Self: 'static + ::std::marker::Sized,
            {
                unsafe {
                    #descriptor_entry
                    #make_dyncast_descriptor
                }
            }

            fn dyncast_attach_vtable(ptr: *const ()) -> *const dyn #trait_ident_with_params
            where
                Self: 'static + ::std::marker::Sized,
            {
                let vtable = ::std::ptr::metadata(
                    ::std::ptr::null::<Self>() as *const dyn #trait_ident_with_params
                );
                ::std::ptr::from_raw_parts(ptr, vtable)
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
                source: &__T
            ) -> ::std::option::Option<&Self> {
                use ::std::any::Any;

                #[cfg(any(
                    target_os = "none",
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "tvos",
                ))]
                extern "Rust" {
                    #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "freebsd"), link_name = #elf_section_start)]
                    #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_start)]
                    static DYNCAST_START: *const ();

                    #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "freebsd"), link_name = #elf_section_stop)]
                    #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_stop)]
                    static DYNCAST_STOP: *const ();
                }

                #generics_type_id

                #make_map

                let key = ::dyncast::private::Key {
                    self_type_id: source.type_id(),
                    generics_type_id,
                };
                let descriptor = map.get(&key)?;

                Some(unsafe {
                    &*(descriptor.attach_vtable())(source as *const _ as *const ())
                })
            }
        }

        #empty_descriptor_entry
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

    Ok(quote! {
        const _: () = {
            #[used]
            static REF_DYNCAST: unsafe fn() = <#self_ty as #trait_path>::__dyncast_descriptor_ref;
        };
    })
}

pub fn expand(item: &mut Item, args: Args) -> Result<TokenStream, Error> {
    match item {
        Item::Trait(item) => expand_trait(item, args),
        Item::Impl(item) => expand_impl(item, args),
    }
}
