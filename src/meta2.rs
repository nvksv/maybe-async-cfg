use proc_macro::{TokenStream};
use proc_macro2::{Span};
use syn::{Error, spanned::Spanned};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ListOfMeta {
    inner: syn::punctuated::Punctuated::<Meta, syn::Token![,]>,
}

impl syn::parse::Parse for ListOfMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self { 
            inner: syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated(input)?,
        })
    }
}

impl From<syn::punctuated::Punctuated::<Meta, syn::Token![,]>> for ListOfMeta {
    fn from(inner: syn::punctuated::Punctuated::<Meta, syn::Token![,]>) -> Self {
        Self {
            inner
        }
    }
}

impl ListOfMeta {
    pub fn len( &self ) -> usize {
        self.inner.len()
    }

    pub fn only_one( &self ) -> Option<&Meta> {
        if self.inner.len() == 1 {
            Some(&self.inner[0])
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ListOfMetaIterator {
    inner: <syn::punctuated::Punctuated::<Meta, syn::Token![,]> as IntoIterator>::IntoIter,
}

impl IntoIterator for ListOfMeta {
    type IntoIter = ListOfMetaIterator;
    type Item = Meta;

    fn into_iter(self) -> Self::IntoIter {
        ListOfMetaIterator {
            inner: self.inner.into_iter(),
        }
    }
}

impl Iterator for ListOfMetaIterator {
    type Item = Meta;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?)
    }

}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ListOfMetaRefIterator<'s> {
    inner: <&'s syn::punctuated::Punctuated::<Meta, syn::Token![,]> as IntoIterator>::IntoIter,
}

impl<'i> IntoIterator for &'i ListOfMeta {
    type IntoIter = <&'i syn::punctuated::Punctuated::<Meta, syn::Token![,]> as IntoIterator>::IntoIter;
    type Item = &'i Meta;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'s> Iterator for ListOfMetaRefIterator<'s> {
    type Item = &'s Meta;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MetaList {
    inner: syn::MetaList,
}

impl From<syn::MetaList> for MetaList {
    fn from(inner: syn::MetaList) -> Self {
        Self {
            inner
        }
    }
}

impl MetaList {
    pub fn parse_as_list_of_meta( &self ) -> syn::Result<ListOfMeta> {
        Ok(
            self.inner
                .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)?
                .into()
        )
    }

    pub fn span( &self ) -> Span {
        self.inner.span()
    }

    pub fn get_ident( &self ) -> Option<&syn::Ident> {
        self.inner.path.get_ident()
    }

    pub fn require_ident( &self, error_message: &str ) -> syn::Result<&syn::Ident> {
        self.get_ident()
            .ok_or_else(|| Error::new(self.span(), error_message))
    }
}



////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum IdentOrLitStrRef<'r> {
    Ident(&'r syn::Ident),
    LitStr(&'r syn::LitStr),
}

////////////////////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Clone)]
pub enum Meta {
    Path(syn::Path),
    List(MetaList),
    NameValue(syn::MetaNameValue),
    Lit(syn::Lit),
    // LitValue(LitWithValue),
    // Empty,
}

impl From<syn::Meta> for Meta {
    fn from(value: syn::Meta) -> Self {
        match value {
            syn::Meta::Path(path) => Self::Path(path),
            syn::Meta::List(list) => Self::List(list.into()),
            syn::Meta::NameValue(name_value) => Self::NameValue(name_value),
        }
    }
}

impl syn::parse::Parse for Meta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // if  input.is_empty() {
        //     return Ok(Self::Empty);
        // }

        if input.peek(syn::Lit) {
            let value = input.parse::<syn::Lit>()?;
            return Ok(Self::Lit(value));
        }

        let value = input.parse::<syn::Meta>()?.into();
        Ok(value)
    }
}

impl Meta {
    pub fn span( &self ) -> Span {
        match self {
            Self::Path(path) => path.span(),
            Self::List(list) => list.span(),
            Self::NameValue(name_value) => name_value.span(),
            Self::Lit(lit) => lit.span(),
            // Self::Empty => None,
        }
    }

    pub fn get_ident( &self ) -> Option<&syn::Ident> {
        match self {
            Self::Path(path) => path.get_ident(),
            Self::List(list) => list.get_ident(),
            Self::NameValue(name_value) => name_value.path.get_ident(),
            Self::Lit(_) => None,
            // Self::Empty => None,
        }
    }

    pub fn get_path_ident( &self ) -> Option<&syn::Ident> {
        match self {
            Self::Path(path) => path.get_ident(),
            _ => None,
        }
    }

    pub fn get_ident_or_lit_str_ref( &self ) -> Option<IdentOrLitStrRef<'_>> {
        match self {
            Self::Path(path) => {
                path.get_ident().map(IdentOrLitStrRef::Ident)
            },
            Self::List(list) => {
                list.get_ident().map(IdentOrLitStrRef::Ident)
            },
            Self::NameValue(name_value) => {
                name_value.path.get_ident().map(IdentOrLitStrRef::Ident)
            },
            Self::Lit(lit) => {
                match lit {
                    syn::Lit::Str(lit_str) => Some(IdentOrLitStrRef::LitStr(lit_str)),
                    _ => None,
                }
            },
            // Self::Empty => None,
        }
    }

    pub fn get_ident_or_lit_str( &self ) -> Option<(String, Span)> {
        self.get_ident_or_lit_str_ref()
            .map(|value| {
                match value {            
                    IdentOrLitStrRef::Ident(ident) => {
                        (ident.to_string(), ident.span())
                    },
                    IdentOrLitStrRef::LitStr(lit_str) => {
                        (lit_str.value(), lit_str.span())
                    },
                }
            })
    }

    pub fn get_any_string_value( &self ) -> Option<(String, Span)> {
        match self {
            Self::Path(_) => {
                None
            },
            Self::List(list) => {
                let inner_list = list.parse_as_list_of_meta().ok()?;
                let inner_meta = inner_list.only_one()?;
                inner_meta.get_ident_or_lit_str()
            },
            Self::NameValue(name_value) => {
                match &name_value.value {
                    syn::Expr::Lit(syn::ExprLit{lit: syn::Lit::Str(lit_str), ..}) => {
                        Some((lit_str.value(), lit_str.span()))
                    },
                    _ => {
                        None
                    },
                }
            },
            Self::Lit(lit) => {
                match lit {
                    syn::Lit::Str(lit_str) => {
                        Some((lit_str.value(), lit_str.span()))
                    },
                    _ => {
                        None
                    },
                }
            },
            // Self::Empty => None,
        }
    }

    // pub fn get_meta_value( &self ) -> Option<Meta> {
    //     match self {
    //         Self::Path(_) => {
    //             None
    //         },
    //         Self::List(list) => {
    //             let inner_list = list.parse_as_list_of_meta().ok()?;
    //             let inner_meta = inner_list.only_one()?;
    //             Some(inner_meta.clone())
    //         },
    //         Self::NameValue(name_value) => {
    //             syn::Meta&name_value.value {
    //                 syn::Expr::Lit(syn::ExprLit{lit: syn::Lit::Str(lit_str), ..}) => {
    //                     Some((lit_str.value(), lit_str.span()))
    //                 },
    //                 _ => {
    //                     None
    //                 },
    //             }
    //         },
    //         Self::Lit(lit) => {
    //             match lit {
    //                 syn::Lit::Str(lit_str) => {
    //                     Some((lit_str.value(), lit_str.span()))
    //                 },
    //                 _ => {
    //                     None
    //                 },
    //             }
    //         },
    //         // Self::Empty => None,
    //     }
    // }

    pub fn require_ident_or_lit_str( &self, error_message: &str ) -> syn::Result<IdentOrLitStrRef<'_>> {
        self.get_ident_or_lit_str_ref()
            .ok_or_else(|| syn::Error::new(self.span(), error_message))
    }

    pub fn require_ident( &self, error_message: &str ) -> syn::Result<&syn::Ident> {
        self.get_ident()
            .ok_or_else(|| syn::Error::new(self.span(), error_message))
    }

    pub fn require_path( &self, error_message: &str ) -> syn::Result<&syn::Path> {
        match self {
            Self::Path(path) => Ok(path),
            _ => Err(Error::new(self.span(), error_message)),
        }
    }

    pub fn require_list( &self, error_message: &str ) -> syn::Result<&MetaList> {
        match self {
            Self::List(list) => Ok(list),
            _ => Err(Error::new(self.span(), error_message)),
        }
    }

    pub fn require_list_of_meta( &self, error_message: &str ) -> syn::Result<ListOfMeta> {
        let list = self.require_list(error_message)?;
        list.parse_as_list_of_meta()
    }

    pub fn require_name_value( &self, error_message: &str ) -> syn::Result<&syn::MetaNameValue> {
        match self {
            Self::NameValue(name_value) => Ok(name_value),
            _ => Err(Error::new(self.span(), error_message)),
        }
    }

    pub fn require_name_value_lit( &self, name_value_error_message: &str, lit_expr_error_message: &str ) -> syn::Result<&syn::ExprLit> {
        let name_value = self.require_name_value(name_value_error_message)?;
        
        match &name_value.value {
            syn::Expr::Lit(lit_expr) => Ok(lit_expr),
            other @ _ => Err(Error::new(other.span(), lit_expr_error_message)),
        }
    }

    pub fn require_name_value_lit_str( &self, name_value_error_message: &str, lit_expr_error_message: &str, lit_str_error_message: &str ) -> syn::Result<&syn::LitStr> {
        let lit_expr = self.require_name_value_lit(name_value_error_message, lit_expr_error_message)?;
        
        match &lit_expr.lit {
            syn::Lit::Str(lit_str) => Ok(lit_str),
            other @ _ => Err(Error::new(other.span(), lit_str_error_message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
