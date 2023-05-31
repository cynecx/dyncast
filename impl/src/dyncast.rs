use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Error, ItemImpl, ItemTrait, TraitItem};

use crate::{args::Args, linker, parse::Item};

enum AsmSectionKind {
    DyncastDescriptorSymbol,
    ZeroData,
}

fn generate_asm_sections(
    trait_ident: &Ident,
    kind: AsmSectionKind,
    is_global: bool,
) -> TokenStream {
    let linux_section = linker::linux::section(trait_ident);
    let macho_section = linker::macho::section(trait_ident);

    let push_linux_section = format!(".pushsection {linux_section}");
    let push_macho_section = format!(".pushsection {macho_section}");

    let (data, bindings, options) = match kind {
        AsmSectionKind::DyncastDescriptorSymbol => (
            ".quad {data}",
            quote!(data = sym Self::dyncast_descriptor,),
            quote!(nomem, preserves_flags, nostack),
        ),
        AsmSectionKind::ZeroData => (".quad 0", quote!(), quote!()),
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
            #data,
            ".popsection",
            #bindings
            options(#options)
        );
        #[cfg(all(any(target_os = "macos", target_os = "ios", target_os = "tvos"), target_pointer_width = "32"))]
        #asm_kind(
            #push_macho_section,
            ".p2align 2, 0",
            #data,
            ".popsection",
            #bindings
            options(#options)
        );
        #[cfg(all(any(target_os = "none", target_os = "linux"), target_pointer_width = "64"))]
        #asm_kind(
            #push_linux_section,
            ".p2align 3, 0",
            #data,
            ".popsection",
            #bindings
            options(#options)
        );
        #[cfg(all(any(target_os = "none", target_os = "linux"), target_pointer_width = "32"))]
        #asm_kind(
            #push_linux_section,
            ".p2align 2, 0",
            #data,
            ".popsection",
            #bindings
            options(#options)
        );
    }
}

pub fn expand_trait(item: &mut ItemTrait, _args: Args) -> Result<TokenStream, Error> {
    if !item.generics.params.is_empty() {
        return Err(Error::new(
            item.generics.span(),
            "generics aren't supported",
        ));
    }

    let trait_ident = &item.ident;

    let dyncast_provider = format!("{}DyncastProvider", &item.ident);
    let dyncast_provider = Ident::new(
        &dyncast_provider,
        Span::call_site().located_at(item.ident.span()),
    );

    let descriptor_entry =
        generate_asm_sections(trait_ident, AsmSectionKind::DyncastDescriptorSymbol, false);
    let empty_descriptor_entry = generate_asm_sections(trait_ident, AsmSectionKind::ZeroData, true);

    let linux_section_start = linker::linux::section_start(trait_ident);
    let linux_section_stop = linker::linux::section_stop(trait_ident);

    let macho_section_start = linker::macho::section_start(trait_ident);
    let macho_section_stop = linker::macho::section_stop(trait_ident);

    let dyncast_descriptor_ref = quote! {
        unsafe fn __dyncast_descriptor_ref()
        where
            Self: 'static + ::std::marker::Sized + #dyncast_provider
        {
            let _ = <Self as #dyncast_provider>::dyncast_descriptor();
        }
    };
    let dyncast_descriptor_ref = syn::parse2::<TraitItem>(dyncast_descriptor_ref).unwrap();

    item.items.push(dyncast_descriptor_ref);

    Ok(quote! {
        unsafe trait #dyncast_provider: #trait_ident {
            fn dyncast_descriptor() -> ::dyncast::private::Descriptor<dyn #trait_ident>
            where
                Self: 'static + ::std::marker::Sized,
            {
                unsafe {
                    #descriptor_entry
                    ::dyncast::private::Descriptor::new(
                        ::std::any::TypeId::of::<Self>(),
                        Self::dyncast_attach_vtable
                    )
                }
            }

            fn dyncast_attach_vtable(ptr: *const ()) -> *const dyn #trait_ident
            where
                Self: 'static + ::std::marker::Sized,
            {
                let vtable = ::std::ptr::metadata(
                    ::std::ptr::null::<Self>() as *const dyn #trait_ident
                );
                ::std::ptr::from_raw_parts(ptr, vtable)
            }
        }

        unsafe impl<T: #trait_ident> #dyncast_provider for T {}

        impl ::dyncast::private::Dyncast for dyn #trait_ident {
            fn dyncast_from<T: ?::std::marker::Sized + ::std::any::Any>(
                source: &T
            ) -> ::std::option::Option<&Self> {
                use ::std::any::Any;

                #[cfg(any(
                    target_os = "none",
                    target_os = "linux",
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "tvos",
                ))]
                extern "Rust" {
                    #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_start)]
                    #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_start)]
                    static DYNCAST_START: *const ();

                    #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_stop)]
                    #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_stop)]
                    static DYNCAST_STOP: *const ();
                }

                let lazy = ::dyncast::private::LazyTypeMap::<dyn #trait_ident>::current();
                let map = unsafe { lazy.get_or_init(&DYNCAST_START, &DYNCAST_STOP) };

                let descriptor = map.get(&source.type_id())?;

                Some(unsafe {
                    &*(descriptor.attach_vtable())(source as *const T as *const ())
                })
            }
        }

        #empty_descriptor_entry
    })
}

pub fn expand_impl(item: &ItemImpl, _args: Args) -> Result<TokenStream, Error> {
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
