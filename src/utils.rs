use crate::params::IdentRecord;

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Expr,
    Ident,
    Meta,
    NestedMeta,
    Token,
};

use quote::ToTokens;

macro_rules! unwrap_or_error {
    ($res:expr) => {
        match $res {
            Ok(p) => p,
            Err(err) => {
                abort!(err)
            }
        }
    };
}
pub(crate) use unwrap_or_error;

macro_rules! unwrap2_or_error {
    ($res:expr) => {
        match $res {
            Ok(p) => p,
            Err(err) => {
                return proc_macro2::TokenStream::from(syn::Error::from(err).to_compile_error());
            }
        }
    };
}

pub(crate) use unwrap2_or_error;

macro_rules! set_error_and_return {
    ($err:expr) => {{
        emit_error!($err);
        return;
    }};
    ($err:expr, $ret:expr) => {{
        emit_error!($err);
        return $ret;
    }};
}
pub(crate) use set_error_and_return;

macro_rules! unwrap_or_set_error_and_return {
    ($res:expr) => {
        match $res {
            Ok(p) => p,
            Err(err) => set_error_and_return!(err),
        }
    };
    ($res:expr, $ret:expr) => {
        match $res {
            Ok(p) => p,
            Err(err) => set_error_and_return!(err, $ret),
        }
    };
}
pub(crate) use unwrap_or_set_error_and_return;

pub(crate) fn make_path(name: &str) -> syn::Path {
    let mut segments = Punctuated::<syn::PathSegment, syn::token::Colon2>::new();
    segments.push_value(syn::PathSegment {
        ident: Ident::new(name, Span::call_site()),
        arguments: syn::PathArguments::None,
    });

    syn::Path {
        leading_colon: None,
        segments,
    }
}

pub(crate) fn make_nestedmeta_namevalue(name: &str, value: &str) -> syn::NestedMeta {
    NestedMeta::Meta(Meta::NameValue(syn::MetaNameValue {
        path: make_path(name),
        eq_token: Token![=](Span::call_site()),
        lit: syn::Lit::Str(syn::LitStr::new(value, Span::call_site())),
    }))
}

pub(crate) fn make_nestedmeta_list(
    name: &str,
    nested: Punctuated<NestedMeta, syn::token::Comma>,
) -> syn::NestedMeta {
    NestedMeta::Meta(Meta::List(syn::MetaList {
        path: make_path(name),
        paren_token: syn::token::Paren(proc_macro2::Span::call_site()),
        nested,
    }))
}

pub(crate) fn make_identrecord(name: &str, ir: &IdentRecord) -> syn::NestedMeta {
    let mut nested = Punctuated::<syn::NestedMeta, syn::token::Comma>::new();
    
    if ir.fn_mode {
        nested.push(syn::NestedMeta::Meta(syn::Meta::Path(make_path("fn"))));
    };

    if ir.use_mode {
        nested.push(syn::NestedMeta::Meta(syn::Meta::Path(make_path("use"))));
    };

    if ir.keep {
        nested.push(syn::NestedMeta::Meta(syn::Meta::Path(make_path("keep"))));
    };

    if let Some(value) = &ir.ident_async {
        if value == name {
            nested.push(syn::NestedMeta::Meta(syn::Meta::Path(make_path("async"))));
        } else {
            nested.push(make_nestedmeta_namevalue("async", value.as_str()));
        }
    };
    if let Some(value) = &ir.ident_sync {
        if value == name {
            nested.push(syn::NestedMeta::Meta(syn::Meta::Path(make_path("sync"))));
        } else {
            nested.push(make_nestedmeta_namevalue("sync", value.as_str()));
        }
    };

    if let Some(idents) = &ir.idents {
        for (key, value) in idents {
            nested.push(make_nestedmeta_namevalue(key.as_str(), value.as_str()));
        }
    };

    if nested.is_empty() {
        syn::NestedMeta::Meta(syn::Meta::Path(make_path(name)))
    } else {
        make_nestedmeta_list(name, nested)
    }
}

pub(crate) fn make_attr_from_str<S: AsRef<str>>(s: S, span: Span) -> syn::Result<syn::Attribute> {
    let stream: TokenStream2 = format!("#[{}]", s.as_ref()).parse()?;
    let mut attrs: Attrs = syn::parse(stream.into())?;
    let attr = match attrs.attrs.len() {
        1 => attrs.attrs.remove(0),
        _ => return Err(syn::Error::new(span, "Expected attribute")),
    };

    Ok(attr)
}

pub(crate) fn make_attr_ts_from_str<S: AsRef<str>>(s: S, span: Span) -> syn::Result<TokenStream2> {
    Ok(make_attr_from_str(s, span)?.to_token_stream())
}

struct Attrs {
    pub attrs: Vec<syn::Attribute>,
}

impl syn::parse::Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Attrs {
            attrs: input.call(syn::Attribute::parse_outer)?,
        })
    }
}

pub struct AttributeArgsInParens {
    _paren: syn::token::Paren,
    pub args: Punctuated<NestedMeta, Token![,]>,
}

impl syn::parse::Parse for AttributeArgsInParens {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _paren: parenthesized!(content in input),
            args: content.parse_terminated(NestedMeta::parse)?,
        })
    }
}

pub struct PunctuatedList {
    pub list: Punctuated<Expr, Comma>,
}

impl Parse for PunctuatedList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PunctuatedList {
            list: Punctuated::<Expr, Comma>::parse_terminated(input)?,
        })
    }
}
