//! Crate not intended for direct use.
//! Use https:://docs.rs/ext-trait instead.
#![allow(nonstandard_style, unused_braces, unused_imports)]

use ::core::{
    mem,
    ops::Not as _,
};
use ::proc_macro::{
    TokenStream,
};
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
    TokenTree as TT,
};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, // Explicitly shadow it
};

///
#[proc_macro_attribute] pub
fn extension (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    extension_impl(attrs.into(), input.into())
    //  .map(|ret| { println!("{}", ret); ret })
        .unwrap_or_else(|err| {
            let mut errors =
                err .into_iter()
                    .map(|err| Error::new(
                        err.span(),
                        format_args!("`#[extension(trait …)]`: {}", err),
                    ))
            ;
            let mut err = errors.next().unwrap();
            errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
}

struct Attrs {
    pub_: Visibility,
    trait_: Token![trait],
    TraitName: Ident,
}

impl Parse for Attrs {
    fn parse (input: ParseStream<'_>)
      -> Result<Attrs>
    {
        Ok(Self {
            pub_: input.parse()?,
            trait_: input.parse()?,
            TraitName: input.parse()?,
        })
    }
}

/// Example
#[cfg(any())]
const _: () = {
    use ::ext_trait::extension;

    #[extension(trait disregard_err)]
    impl<T, E> Result<T, E> {
        fn disregard_err(self) -> Option<T> { self }
    }
};

fn extension_impl (
    attrs: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    let trait_def_span = attrs.span();
    let Attrs { pub_, trait_, TraitName } = parse2(attrs)?;
    let ref mut item_impl: ItemImpl = parse2(input)?;
    let attrs = mem::take(&mut item_impl.attrs);
    item_impl.attrs = attrs_to_forward_to_impl_block(&attrs);
    let (intro_generics, fwd_generics, where_clause) = item_impl.generics.split_for_impl();
    match Option::replace(
        &mut item_impl.trait_,
        (None, parse_quote!( #TraitName #fwd_generics ), <_>::default()),
    )
    {
        | Some((_, _, extraneous_for)) => return Err(Error::new_spanned(
            extraneous_for,
            "expected inherent `impl<…> Type<…>` syntax",
        )),
        | _ => {},
    }
    let ref item_impl = item_impl;
    let each_entry = item_impl.items.iter().map(|it| Ok(match it {
        // We don't deny `pub_` and `default_` annotations *directly*:
        // instead, we forward their extraneous presence to the `trait`
        // definition, so as to trigger a grammar error from the following
        // rust parser pass, which ought to yield a way nicer error message.
        | ImplItem::Const(ImplItemConst {
            vis: pub_,
            defaultness: default_,
            const_token: const_,
            ident: CONST_NAME @ _,
            ty: Ty @ _,
            attrs,
            ..
        }) => {
            let attrs = attrs_to_forward_to_trait_items(attrs);
            quote!(
                #(#attrs)*
                #pub_
                #default_
                #const_ #CONST_NAME: #Ty;
            )
        }

        | ImplItem::Method(ImplItemMethod {
            vis: pub_,
            defaultness: default_,
            sig,
            attrs,
            ..
        }) => {
            let attrs = attrs_to_forward_to_trait_items(attrs);
            let mut sig = sig.clone();
            sig.inputs.iter_mut().for_each(|fn_arg| match fn_arg {
                | FnArg::Receiver(Receiver { reference, mutability, .. }) => {
                    if reference.is_none() {
                        *mutability = None;
                    }
                },
                | FnArg::Typed(PatType { pat, .. }) => {
                    *pat = parse_quote!( _ );
                },
            });
            quote!(
                #(#attrs)*
                #pub_
                #default_
                #sig;
            )
        },

        | ImplItem::Type(ImplItemType {
            vis: pub_,
            defaultness: default_,
            type_token: type_,
            ident: TypeName @ _,
            generics,
            semi_token: SEMICOLON @ _,
            attrs,
            ..
        }) => {
            let attrs = attrs_to_forward_to_trait_items(attrs);
            quote! (
                #(#attrs)*
                #pub_
                #default_
                #type_ #TypeName #generics
                :
                    ?::ext_trait::__::core::marker::Sized
                #SEMICOLON
            )
        },

        | _ => return Err(Error::new_spanned(it, "unsupported `impl` entry")),
    })).collect::<Result<Vec<_>>>()?;
    let ItemImpl { self_ty: ref Receiver, .. } = *item_impl;
    let docs_addendum = format!(r#"

This is an extension trait for the following impl:
```rust ,ignore
#[extension(pub trait {TraitName})]
impl{intro_generics} for {Receiver}
{maybe_where}{where_clauses}
```"#,
        intro_generics = intro_generics.to_token_stream(),
        Receiver = Receiver.to_token_stream(),
        maybe_where = if where_clause.is_some() { "where" } else { "" },
        where_clauses =
            where_clause
                .iter()
                .flat_map(|w| w.predicates.iter().map(|p| format!("\n    {}", p.to_token_stream())))
                .collect::<String>()
        ,
    );
    Ok(quote_spanned!(trait_def_span=>
        #(#attrs)*
        #[doc = #docs_addendum]
        #[allow(nonstandard_style)]
        #pub_
        #trait_ #TraitName #intro_generics
        #where_clause
        {
            #(#each_entry)*
        }

        #item_impl
    ))
}

fn attrs_to_forward_to_impl_block(
    trait_attrs: &[Attribute],
) -> Vec<Attribute>
{
    const IMPL_BLOCK_ATTRS_ALLOW_LIST: &[&str] = &[
        "doc",
        "allow",
        "warn",
        "deny",
        "forbid",
        "async_trait",
    ];

    trait_attrs.iter().filter(|attr| IMPL_BLOCK_ATTRS_ALLOW_LIST.iter().any(|ident| {
        attr.path.is_ident(ident)
    })).cloned().collect()
}

fn attrs_to_forward_to_trait_items(
    impl_block_assoc_item_attrs: &[Attribute],
) -> Vec<&Attribute>
{
    const TRAIT_ITEMS_ATTRS_DENY_LIST: &[&str] = &[
        "inline",
    ];

    impl_block_assoc_item_attrs.iter().filter(|attr| TRAIT_ITEMS_ATTRS_DENY_LIST.iter().all(|ident| {
        attr.path.is_ident(ident).not()
    })).collect()
}
