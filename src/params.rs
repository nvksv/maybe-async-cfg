use std::collections::HashMap;
#[allow(unused_imports)]
use std::iter::FromIterator;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, AttributeArgs, Ident, Lit,
    LitStr, Meta, MetaNameValue, NestedMeta,
};

use crate::{
    macros::ConvertMode, utils::AttributeArgsInParens, DEFAULT_CRATE_NAME, STANDARD_MACROS,
};

#[derive(Debug, Clone, Copy)]
pub enum MacroParameterActionKind {
    Sync,
    Async,
}

impl MacroParameterActionKind {
    fn from_str<S: AsRef<str>>(s: S) -> Option<Self> {
        match s.as_ref() {
            "sync" => Some(MacroParameterActionKind::Sync),
            "async" => Some(MacroParameterActionKind::Async),
            _ => None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            MacroParameterActionKind::Sync => "sync",
            MacroParameterActionKind::Async => "async",
        }
    }
}

use crate::utils::*;

#[derive(Debug, Clone)]
pub struct MacroParameterAction {
    pub kind: MacroParameterActionKind,
    pub params: MacroParameters,
}

#[derive(Debug, Clone)]
pub struct MacroParameters {
    disable: bool,
    key: Option<String>,
    // settings
    prefix: Option<String>,
    idents: HashMap<String, IdentRecord>,
    send: Option<bool>,
    keep_self: bool,
    // groups
    cfg: Option<Meta>,
    outer_attrs: Punctuated<NestedMeta, Comma>,
    inner_attrs: Punctuated<NestedMeta, Comma>,
    drop_attrs: Vec<String>,
    replace_features: HashMap<String, String>,
    // actions
    pub actions: Vec<MacroParameterAction>,
}

macro_rules! lit_str {
    ($lit:ident, $obj:expr, $fn:ident, $msg:expr) => {
        match $lit {
            syn::Lit::Str(str_val) => $obj.$fn(str_val.value())?,
            _ => return Err(syn::Error::new_spanned($lit.to_token_stream(), $msg)),
        }
    };
}

macro_rules! lit_meta {
    ($lit:ident, $meta:expr, $obj:expr, $fn:ident, $msg:expr) => {
        match $lit {
            syn::Lit::Str(_) => $obj.$fn($meta)?,
            _ => return Err(syn::Error::new_spanned($lit.to_token_stream(), $msg)),
        }
    };
}

impl MacroParameters {
    #[allow(dead_code)]
    pub fn new() -> Self {
        MacroParametersBuilder::new().build().unwrap()
    }

    fn from_args<'i>(args: impl IntoIterator<Item = &'i NestedMeta>) -> syn::Result<Self> {
        let mut builder = MacroParametersBuilder::new();

        for arg in args {
            match arg {
                syn::NestedMeta::Meta(meta) => match meta {
                    syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) => {
                        let name = path
                            .get_ident()
                            .ok_or(syn::Error::new_spanned(
                                path.to_token_stream(),
                                "Expected name",
                            ))?
                            .to_string();
                        match name.as_str() {
                            "key" => lit_str!(lit, builder, key, "Expected string literal"),
                            "prefix" => lit_str!(lit, builder, prefix, "Expected string literal"),
                            "send" => lit_str!(lit, builder, send, "Expected string literal"),
                            "feature" => {
                                lit_meta!(lit, meta, builder, feature, "Expected string literal")
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    meta.to_token_stream(),
                                    format!("Wrong name for name-value pair: {}", &name),
                                ))
                            }
                        }
                    }
                    syn::Meta::List(list) => {
                        let name = list
                            .path
                            .get_ident()
                            .ok_or(syn::Error::new_spanned(
                                list.path.to_token_stream(),
                                "Expected name",
                            ))?
                            .to_string();
                        match name.as_str() {
                            "cfg" => match list.nested.len() {
                                0 => {
                                    return Err(syn::Error::new_spanned(
                                        meta.to_token_stream(),
                                        "Expected condition",
                                    ))
                                }
                                1 => {
                                    let first = list.nested.first().unwrap();
                                    match first {
                                        NestedMeta::Meta(first_meta) => builder.cfg(first_meta)?,
                                        _ => {
                                            return Err(syn::Error::new_spanned(
                                                meta.to_token_stream(),
                                                "Expected condition",
                                            ))
                                        }
                                    }
                                }
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        meta.to_token_stream(),
                                        "Expected condition",
                                    ))
                                }
                            },
                            "idents" => MacroParametersBuilder::idents(
                                &mut builder.params.idents,
                                &list.nested,
                            )?,
                            "any" | "all" | "not" => builder.cfg(meta)?,
                            "outer" => builder.outer_attrs(&list.nested)?,
                            "inner" => builder.inner_attrs(&list.nested)?,
                            "replace_feature" => builder.replace_feature(&list.nested)?,
                            "drop_attrs" => builder.drop_attrs(&list.nested)?,
                            name @ _ => builder.action_or_inner_attr(name, &list.nested, meta)?,
                        }
                    }
                    syn::Meta::Path(path) => {
                        if path.is_ident("disable") {
                            builder.disable()
                        } else if path.is_ident("keep_self") {
                            builder.keep_self()
                        } else {
                            builder.inner_attr(meta)?
                        }
                    }
                },
                syn::NestedMeta::Lit(lit) => {
                    lit_meta!(lit, lit, builder, inner_attr_str, "Expected string literal")
                }
            }
        }

        builder.build()
    }

    pub fn from_tokens(tokens: TokenStream) -> syn::Result<Self> {
        let args = match syn::parse_macro_input::parse::<AttributeArgs>(tokens) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };

        Self::from_args(&args)
    }

    pub fn from_tokens_in_parens(tokens: TokenStream) -> syn::Result<Self> {
        let aip = match syn::parse_macro_input::parse::<AttributeArgsInParens>(tokens) {
            Ok(a) => a,
            Err(e) => {
                return Err(e);
            }
        };

        Self::from_args(&aip.args)
    }

    pub fn to_nestedmeta(&self) -> Punctuated<syn::NestedMeta, syn::token::Comma> {
        let mut args = Punctuated::<syn::NestedMeta, syn::token::Comma>::new();

        if self.disable {
            args.push(NestedMeta::Meta(Meta::Path(make_path("disable"))));
        }

        if self.keep_self {
            args.push(NestedMeta::Meta(Meta::Path(make_path("keep_self"))));
        }

        if let Some(key) = &self.key {
            args.push(make_nestedmeta_namevalue("key", key.as_str()));
        }

        if let Some(prefix) = &self.prefix {
            args.push(make_nestedmeta_namevalue("prefix", prefix.as_str()));
        }

        if let Some(send) = &self.send {
            args.push(make_nestedmeta_namevalue(
                "prefix",
                if *send { "Send" } else { "?Send" },
            ));
        }

        if let Some(cfg) = &self.cfg {
            let mut nested = Punctuated::new();
            nested.push(NestedMeta::Meta(cfg.clone()));
            args.push(make_nestedmeta_list("cfg", nested));
        }

        if !self.outer_attrs.is_empty() {
            args.push(make_nestedmeta_list("outer", self.outer_attrs.clone()));
        }

        if !self.inner_attrs.is_empty() {
            args.push(make_nestedmeta_list("inner", self.inner_attrs.clone()));
        }

        if !self.idents.is_empty() {
            let mut nested = Punctuated::<syn::NestedMeta, syn::token::Comma>::new();
            for (name, value) in &self.idents {
                nested.push(make_identrecord(name.as_str(), value));
            }
            let arg = make_nestedmeta_list("idents", nested);
            args.push(arg);
        }

        if !self.drop_attrs.is_empty() {
            let mut nested = Punctuated::<syn::NestedMeta, syn::token::Comma>::new();
            for name in &self.drop_attrs {
                nested.push(NestedMeta::Meta(Meta::Path(make_path(name.as_str()))));
            }
            let arg = make_nestedmeta_list("drop_attrs", nested);
            args.push(arg);
        }

        if !self.replace_features.is_empty() {
            for (name, value) in &self.replace_features {
                let mut inner = Punctuated::<syn::NestedMeta, syn::token::Comma>::new();
                inner.push(NestedMeta::Lit(Lit::Str(LitStr::new(
                    name.as_str(),
                    Span::call_site(),
                ))));
                inner.push(NestedMeta::Lit(Lit::Str(LitStr::new(
                    value.as_str(),
                    Span::call_site(),
                ))));
                let arg = make_nestedmeta_list("replace_feature", inner);
                args.push(arg);
            }
        }

        for action in &self.actions {
            let (name, nested) = match action.kind {
                MacroParameterActionKind::Sync | MacroParameterActionKind::Async => {
                    (action.kind.to_str(), action.params.to_nestedmeta())
                }
            };

            let arg = make_nestedmeta_list(name, nested);
            args.push(arg);
        }

        args
    }

    pub fn extend_tokenstream2_with_cfg_outer_attrs(
        &self,
        ts: &mut TokenStream2,
    ) -> syn::Result<()> {
        if let Some(cfg_cond) = &self.cfg {
            let cfg_ts = cfg_cond.into_token_stream();
            ts.extend(quote!(#[cfg(#cfg_ts)]));
        };

        for attr in &self.outer_attrs {
            match attr {
                NestedMeta::Meta(_) => {
                    let attr_ts = attr.into_token_stream();
                    ts.extend(quote!(#[#attr_ts]));
                }
                NestedMeta::Lit(syn::Lit::Str(s)) => {
                    let attr_ts = make_attr_ts_from_str(s.value(), attr.span())?;
                    ts.extend(attr_ts);
                }
                _ => {
                    unreachable!()
                }
            };
        }

        Ok(())
    }

    pub fn extend_tokenstream2_with_inner_attrs(&self, ts: &mut TokenStream2) -> syn::Result<()> {
        for attr in &self.inner_attrs {
            match attr {
                NestedMeta::Meta(_) => {
                    let attr_ts = attr.into_token_stream();
                    ts.extend(quote!(#[#attr_ts]));
                }
                NestedMeta::Lit(syn::Lit::Str(s)) => {
                    let attr_ts = make_attr_ts_from_str(s.value(), attr.span())?;
                    ts.extend(attr_ts);
                }
                _ => {
                    unreachable!()
                }
            };
        }

        Ok(())
    }

    pub fn to_tokens(&self) -> TokenStream2 {
        self.to_nestedmeta().to_token_stream()
    }

    pub fn default_ident_record(&self, fn_mode: bool) -> IdentRecord {
        IdentRecord::with_fn_mode( fn_mode )
    }

    pub fn apply_parent(child: &mut MacroParameters, parent: &MacroParameters) -> syn::Result<()> {
        if parent.disable {
            child.disable = true;
        }

        if parent.keep_self {
            child.keep_self = true;
        }

        if !parent.idents.is_empty() {
            child.idents.extend(parent.idents.clone());
        }

        if !parent.drop_attrs.is_empty() {
            let mut new_drop_attrs = parent.drop_attrs.clone();
            new_drop_attrs.extend_from_slice(&child.drop_attrs);
            child.drop_attrs = new_drop_attrs;
        }

        if !parent.replace_features.is_empty() {
            child
                .replace_features
                .extend(parent.replace_features.clone());
        }

        Ok(())
    }

    pub fn disable_get(&self) -> bool {
        self.disable
    }

    pub fn keep_self_get(&self) -> bool {
        self.keep_self
    }

    pub fn idents_add<S: AsRef<str>>(&mut self, name: S, fn_mode: bool) {
        if self.idents.get(name.as_ref()).is_none() {
            let ir = self.default_ident_record(fn_mode);
            self.idents.insert(name.as_ref().to_string(), ir);
        }
    }

    pub fn key_get<'s>(&'s self) -> Option<&'s str> {
        self.key.as_ref().map(|s| s.as_str())
    }

    pub fn prefix_set(&mut self, prefix: String) {
        self.prefix = Some(prefix);
    }

    pub fn prefix_get(&self) -> &str {
        self.prefix
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(DEFAULT_CRATE_NAME)
    }

    pub fn send_get(&self) -> Option<bool> {
        self.send
    }

    pub fn idents_get<'s, S: AsRef<str>>(&'s self, name: S) -> Option<&'s IdentRecord> {
        self.idents.get(name.as_ref())
    }

    pub fn replace_features_is_empty(&self) -> bool {
        self.replace_features.is_empty()
    }
    pub fn replace_features_get<'s, S: AsRef<str>>(&'s self, name: S) -> Option<&'s str> {
        self.replace_features.get(name.as_ref()).map(|s| s.as_str())
    }

    pub fn drop_attrs_is_empty(&self) -> bool {
        self.drop_attrs.is_empty()
    }
    pub fn drop_attrs_contains(&self, name: &String) -> bool {
        self.drop_attrs.contains(name)
    }

    pub fn is_our_attr(&self, attr: &Attribute) -> Option<String> {
        if attr.style == syn::AttrStyle::Outer {
            if attr.path.leading_colon.is_none() && attr.path.segments.len() == 2 {
                let first_segment = &attr.path.segments[0];
                let last_segment = &attr.path.segments[1];
                if first_segment.arguments == syn::PathArguments::None
                    && last_segment.arguments == syn::PathArguments::None
                {
                    let first = first_segment.ident.to_string();
                    let last = last_segment.ident.to_string();

                    if let Some(prefix) = &self.prefix {
                        if &first == prefix {
                            return Some(last);
                        }
                    } else {
                        return Some(last);
                    }
                }
            }
        };

        None
    }

    pub fn make_self_path(&self, name: &str) -> syn::Path {
        let mut segments = Punctuated::<syn::PathSegment, syn::token::Colon2>::new();
        segments.push_value(syn::PathSegment {
            ident: Ident::new(
                self.prefix
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or(self.prefix_get()),
                Span::call_site(),
            ),
            arguments: syn::PathArguments::None,
        });
        segments.push_punct(syn::Token![::](Span::call_site()));
        segments.push_value(syn::PathSegment {
            ident: Ident::new(name, Span::call_site()),
            arguments: syn::PathArguments::None,
        });

        syn::Path {
            leading_colon: None,
            segments,
        }
    }

    pub fn standard_macros<'s>(&'s self) -> &[&'s str] {
        STANDARD_MACROS
    }
}

#[derive(Debug, Clone)]
pub struct IdentRecord {
    pub fn_mode: bool,
    pub use_mode: bool,
    pub keep: bool,
    pub ident_sync: Option<String>,
    pub ident_async: Option<String>,
    pub idents: Option<HashMap<String, String>>,
}

impl IdentRecord {
    pub fn new() -> Self {
        Self {
            fn_mode: false,
            use_mode: false,
            keep: false,
            ident_sync: None,
            ident_async: None,
            idents: None,
        }
    }

    pub fn with_fn_mode( fn_mode: bool ) -> Self {
        Self {
            fn_mode,
            use_mode: false,
            keep: false,
            ident_sync: None,
            ident_async: None,
            idents: None,
        }
    }

    pub fn ident_add_suffix(&self, ident: &Ident, convert_mode: ConvertMode, version_name: Option<&str>) -> Ident {
        if self.keep {
            return ident.clone();
        }

        if let Some(version_name) = version_name {
            if let Some(idents) = self.idents.as_ref() {
                if let Some(value) = idents.get(version_name) {
                    return Ident::new(value, ident.span());
                }
            }
        }

        match convert_mode {
            ConvertMode::ToSync => {
                if let Some(name) = &self.ident_sync {
                    return Ident::new(&name, ident.span());
                }
            }
            ConvertMode::ToAsync => {
                if let Some(name) = &self.ident_async {
                    return Ident::new(&name, ident.span());
                }
            }
        };

        let suffix = match (self.fn_mode, convert_mode) {
            (false, ConvertMode::ToAsync) => "Async",
            (false, ConvertMode::ToSync) => "Sync",
            (true, ConvertMode::ToAsync) => "_async",
            (true, ConvertMode::ToSync) => "_sync",
        };

        // TODO: not sure if ident.span() is the way to go
        Ident::new(&format!("{}{}", ident, suffix), ident.span())
    }
}

#[derive(Debug)]
pub struct MacroParametersBuilder {
    params: MacroParameters,
}

impl MacroParametersBuilder {
    pub fn new() -> Self {
        Self {
            params: MacroParameters {
                disable: false,
                key: None,
                prefix: None,
                idents: HashMap::new(),
                keep_self: false,
                send: None,
                cfg: None,
                outer_attrs: Punctuated::new(),
                inner_attrs: Punctuated::new(),
                drop_attrs: vec![],
                replace_features: HashMap::new(),
                actions: vec![],
            },
        }
    }

    pub fn key(&mut self, key: String) -> syn::Result<()> {
        self.params.key = Some(key);
        Ok(())
    }

    pub fn disable(&mut self) {
        self.params.disable = true;
    }

    pub fn keep_self(&mut self) {
        self.params.keep_self = true;
    }

    pub fn prefix(&mut self, prefix: String) -> syn::Result<()> {
        self.params.prefix = Some(prefix);
        Ok(())
    }

    pub fn idents(
        idents: &mut HashMap<String, IdentRecord>,
        list: &Punctuated<NestedMeta, Comma>,
    ) -> syn::Result<()> {
        for nm in list {
            match nm {
                NestedMeta::Meta(Meta::Path(path)) => {
                    let ident = path
                        .get_ident()
                        .ok_or(syn::Error::new_spanned(
                            nm.to_token_stream(),
                            "Expected ident, but not complex path",
                        ))?
                        .to_string();
                    let ir = IdentRecord::new();
                    idents.insert(ident, ir);
                }
                NestedMeta::Meta(Meta::List(syn::MetaList { path, nested, .. })) => {
                    let ident = path
                        .get_ident()
                        .ok_or(syn::Error::new_spanned(
                            nm.to_token_stream(),
                            "Expected ident, but not complex path",
                        ))?
                        .to_string();
                    let mut ir = IdentRecord::new();
                    for inm in nested {
                        match inm {
                            NestedMeta::Meta(Meta::Path(path)) => {
                                let iname = path
                                    .get_ident()
                                    .ok_or(syn::Error::new_spanned(
                                        nm.to_token_stream(),
                                        "Expected ident, but not complex path",
                                    ))?
                                    .to_string();
                                match iname.as_str() {
                                    "fn" => {
                                        ir.fn_mode = true;
                                    }
                                    "use" => {
                                        ir.use_mode = true;
                                    }
                                    "keep" => {
                                        ir.keep = true;
                                    }
                                    "sync" => {
                                        ir.ident_sync = Some(ident.clone());
                                    }
                                    "async" => {
                                        ir.ident_async = Some(ident.clone());
                                    }
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            nm.to_token_stream(),
                                            "Expected fn, use, keep, sync, async",
                                        ))
                                    }
                                }
                            }
                            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                path,
                                lit: syn::Lit::Str(lit),
                                ..
                            })) => {
                                let iname = path
                                    .get_ident()
                                    .ok_or(syn::Error::new_spanned(
                                        nm.to_token_stream(),
                                        "Expected ident, but not complex path",
                                    ))?
                                    .to_string();
                                let ivalue = lit.value();
                                match iname.as_str() {
                                    "sync" => {
                                        ir.ident_sync = Some(ivalue);
                                    }
                                    "async" => {
                                        ir.ident_async = Some(ivalue);
                                    }
                                    _ => {
                                        let idents = ir.idents.get_or_insert_with(|| HashMap::new());
                                        idents.insert(iname, ivalue);
                                    }
                                }
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    nm.to_token_stream(),
                                    "Expected fn, sync = \"ident\", or async = \"ident\"",
                                ))
                            }
                        }
                    }
                    idents.insert(ident, ir);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        nm.to_token_stream(),
                        "Expected name = \"value\" pair",
                    ))
                }
            }
        }

        Ok(())
    }

    pub fn send(&mut self, send: String) -> syn::Result<()> {
        self.params.send = Some(match send.as_str() {
            "" | "Send" | "true" => true,
            "?Send" | "false" => false,
            _ => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Only accepts `Send` or `?Send`",
                ));
            }
        });

        Ok(())
    }

    pub fn feature(&mut self, meta: &Meta) -> syn::Result<()> {
        self.cfg(meta)
    }

    pub fn cfg(&mut self, meta: &Meta) -> syn::Result<()> {
        self.params.cfg = Some(meta.clone());
        Ok(())
    }

    pub fn outer_attrs(&mut self, list: &Punctuated<NestedMeta, Comma>) -> syn::Result<()> {
        if self.params.outer_attrs.is_empty() {
            self.params.outer_attrs = list.clone();
        } else {
            self.params.outer_attrs.extend(list.into_iter().cloned());
        }
        Ok(())
    }

    pub fn inner_attr_str(&mut self, lit: &syn::Lit) -> syn::Result<()> {
        self.params.inner_attrs.push(NestedMeta::Lit(lit.clone()));
        Ok(())
    }

    pub fn inner_attr(&mut self, meta: &Meta) -> syn::Result<()> {
        self.params.inner_attrs.push(NestedMeta::Meta(meta.clone()));
        Ok(())
    }

    pub fn action_or_inner_attr(
        &mut self,
        name: &str,
        list: &Punctuated<NestedMeta, Comma>,
        meta: &Meta,
    ) -> syn::Result<()> {
        if let Some(kind) = MacroParameterActionKind::from_str(name) {
            self.action(kind, list)?;
        } else {
            self.params.inner_attrs.push(NestedMeta::Meta(meta.clone()));
        };
        Ok(())
    }

    pub fn inner_attrs(&mut self, list: &Punctuated<NestedMeta, Comma>) -> syn::Result<()> {
        if self.params.inner_attrs.is_empty() {
            self.params.inner_attrs = list.clone();
        } else {
            self.params.inner_attrs.extend(list.into_iter().cloned());
        }
        Ok(())
    }

    pub fn action(
        &mut self,
        kind: MacroParameterActionKind,
        list: &Punctuated<NestedMeta, Comma>,
    ) -> syn::Result<()> {
        let inner = MacroParameters::from_args(list)?;
        self.params.actions.push(MacroParameterAction {
            kind,
            params: inner,
        });
        Ok(())
    }

    pub fn drop_attrs(&mut self, meta: &Punctuated<NestedMeta, Comma>) -> syn::Result<()> {
        for nm in meta {
            match nm {
                NestedMeta::Meta(Meta::Path(path)) => {
                    let name = path
                        .get_ident()
                        .ok_or(syn::Error::new_spanned(
                            path.to_token_stream(),
                            "Expected ident",
                        ))?
                        .to_string();
                    self.params.drop_attrs.push(name);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        nm.to_token_stream(),
                        "Expected list of idents",
                    ))
                }
            }
        }
        Ok(())
    }

    pub fn replace_feature(&mut self, meta: &Punctuated<NestedMeta, Comma>) -> syn::Result<()> {
        match meta.len() {
            2 => {
                let prev = match &meta[0] {
                    NestedMeta::Lit(Lit::Str(lit)) => lit.value(),
                    nm @ _ => {
                        return Err(syn::Error::new_spanned(
                            nm.to_token_stream(),
                            "Expected string literal",
                        ))
                    }
                };
                let new = match &meta[1] {
                    NestedMeta::Lit(Lit::Str(lit)) => lit.value(),
                    nm @ _ => {
                        return Err(syn::Error::new_spanned(
                            nm.to_token_stream(),
                            "Expected string literal",
                        ))
                    }
                };

                self.params.replace_features.insert(prev, new);
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    meta.to_token_stream(),
                    "Expected two string literals",
                ))
            }
        };

        Ok(())
    }

    pub fn build(mut self) -> syn::Result<MacroParameters> {
        let mut actions = std::mem::replace(&mut self.params.actions, vec![]);

        for action in &mut actions {
            MacroParameters::apply_parent(&mut action.params, &self.params)?;

            if action.params.key.is_none() {
                action.params.key = Some(action.kind.to_str().to_string());
            }
        }

        self.params.actions = actions;

        Ok(self.params)
    }
}
