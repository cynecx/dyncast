use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, Error, GenericParam, Generics, ItemImpl, ItemTrait, Token,
    TraitItem, WherePredicate,
};

use crate::{
    args::Args,
    linker::{self, SectionNameArgs},
    parse::Item,
};

#[derive(Debug, Clone, Copy)]
enum AsmSectionKind {
    DyncastDescriptorSymbol,
    ZeroData,
}

fn generate_asm_sections(
    section_name_args: SectionNameArgs<'_>,
    kind: AsmSectionKind,
    is_global: bool,
) -> TokenStream {
    let elf_section = linker::elf::section(section_name_args);
    let macho_section = linker::macho::section(section_name_args);

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
    let Some(global_id) = &args.global_id else {
        return Err(Error::new(args.end_span, "global_id must be set"));
    };

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

    let inner_trait_ident = Ident::new(&format!("Inner{trait_ident}"), Span::call_site());

    let dyncast_provider = format!("{}DyncastProvider", &item.ident);
    let dyncast_provider = Ident::new(
        &dyncast_provider,
        Span::call_site().located_at(item.ident.span()),
    );
    let dyncast_provider_with_params =
        quote!(#dyncast_provider #generics_lt #generics_params_pass #generics_gt);

    let section_name_args = SectionNameArgs {
        name: trait_ident,
        global_id,
        seed: args.seed,
    };

    let descriptor_entry = generate_asm_sections(
        section_name_args,
        AsmSectionKind::DyncastDescriptorSymbol,
        false,
    );
    let empty_descriptor_entry =
        generate_asm_sections(section_name_args, AsmSectionKind::ZeroData, true);

    let elf_section_start = linker::elf::section_start(section_name_args);
    let elf_section_stop = linker::elf::section_stop(section_name_args);

    let macho_section_start = linker::macho::section_start(section_name_args);
    let macho_section_stop = linker::macho::section_stop(section_name_args);

    let generics_type_id = if has_generics {
        quote! {
            let __generics_type_id = Some(
                ::dyncast::private::TypeId::of::<(#generics_params_pass)>()
            );
        }
    } else {
        quote! {
            let __generics_type_id = None;
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
            let __lazy = unsafe {
                ::dyncast::private::LazyTypeMap::<dyn #inner_trait_ident>::current()
            };
            let __map = unsafe { __lazy.get_or_init(&DYNCAST_START, &DYNCAST_STOP) };
        }
    } else {
        quote! {
            trait #inner_trait_ident: 'static {}
            static __LAZY_MAP: ::dyncast::private::LazyTypeMap::<dyn #inner_trait_ident> =
                ::dyncast::private::LazyTypeMap::new();
            let __map = unsafe { __LAZY_MAP.get_or_init(&DYNCAST_START, &DYNCAST_STOP) };
        }
    };

    let dyncast_descriptor_ref = quote! {
        #[doc(hidden)]
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
        #[doc(hidden)]
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

                let __key = ::dyncast::private::Key {
                    self_type_id: __source.type_id(),
                    generics_type_id: __generics_type_id,
                };
                let __descriptor = __map.get(&__key)?;

                Some(unsafe {
                    &*(__descriptor.attach_vtable_fn())(__source as *const _ as *const ())
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
