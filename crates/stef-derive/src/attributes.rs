use proc_macro2::{Ident, Span};
use syn::{
    meta::ParseNestedMeta, parenthesized, punctuated::Punctuated, token::Comma, Attribute, Expr,
    LitStr, Path,
};

macro_rules! bail {
    ($tokens:expr, $($arg:tt)*) => {
        return Err(syn::Error::new($tokens, format!($($arg)*)))
    };
}

pub struct StructAttributes {
    pub msg: LitStr,
    pub code: Path,
    pub help: Punctuated<Expr, Comma>,
    pub rename: Option<Ident>,
}

impl StructAttributes {
    pub fn parse(attrs: &[Attribute], span: Span) -> syn::Result<Self> {
        let mut msg = None;
        let mut code = None;
        let mut help = None;

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("err")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("msg") {
                    let content;
                    parenthesized!(content in meta.input);
                    msg = Some(content.parse()?);
                } else if meta.path.is_ident("code") {
                    let content;
                    parenthesized!(content in meta.input);
                    code = Some(content.parse()?);
                } else if meta.path.is_ident("help") {
                    let content;
                    parenthesized!(content in meta.input);
                    help = Some(Punctuated::parse_terminated(&content)?);
                }

                Ok(())
            })?;
        }

        let mut rename = None;

        if let Some(attr) = attrs
            .iter()
            .rev()
            .find(|attr| attr.path().is_ident("rename"))
        {
            rename = Some(attr.parse_args()?);
        }

        let Some(msg) = msg else {
            bail!(span, "missing `msg` attribute")
        };
        let Some(code) = code else {
            bail!(span, "missing `code` attribute")
        };
        let Some(help) = help else {
            bail!(span, "missing `help` attribute")
        };

        Ok(Self {
            msg,
            code,
            help,
            rename,
        })
    }
}

pub struct EnumAttributes {
    pub rename: Option<Ident>,
}

impl EnumAttributes {
    pub fn parse(attrs: &[Attribute], _span: Span) -> syn::Result<Self> {
        let mut rename = None;

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("rename")) {
            rename = Some(attr.parse_args()?);
        }

        Ok(Self { rename })
    }
}

pub enum VariantAttributes {
    Error {
        msg: LitStr,
        code: Path,
        help: Punctuated<Expr, Comma>,
    },
    External,
    Forward,
}

impl VariantAttributes {
    pub fn parse(attrs: &[Attribute], span: Span) -> syn::Result<Self> {
        for attr in attrs {
            if attr.path().is_ident("err") {
                let mut msg = None;
                let mut code = None;
                let mut help = None;

                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("msg") {
                        let content;
                        parenthesized!(content in meta.input);
                        msg = Some(content.parse()?);
                    } else if meta.path.is_ident("code") {
                        let content;
                        parenthesized!(content in meta.input);
                        code = Some(content.parse()?);
                    } else if meta.path.is_ident("help") {
                        let content;
                        parenthesized!(content in meta.input);
                        help = Some(Punctuated::parse_terminated(&content)?);
                    }

                    Ok(())
                })?;

                let Some(msg) = msg else {
                    bail!(span, "missing `msg` attribute")
                };
                let Some(code) = code else {
                    bail!(span, "missing `code` attribute")
                };
                let Some(help) = help else {
                    bail!(span, "missing `help` attribute")
                };

                return Ok(Self::Error { msg, code, help });
            } else if attr.path().is_ident("external") {
                return Ok(Self::External);
            } else if attr.path().is_ident("forward") {
                return Ok(Self::Forward);
            }
        }

        bail!(
            span,
            "none of the `err`, `external` or `forward` attributes found"
        )
    }
}

pub struct FieldAttributes {
    pub label: (bool, Option<LitStr>),
}

impl FieldAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        Ok(
            if let FieldAttributesParser { label: Some(label) } =
                FieldAttributesParser::parse_all(attrs)?
            {
                Some(Self { label })
            } else {
                None
            },
        )
    }
}

#[derive(Default)]
struct FieldAttributesParser {
    label: Option<(bool, Option<LitStr>)>,
}

impl FieldAttributesParser {
    fn parse_all(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut value = Self::default();

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("err")) {
            attr.parse_nested_meta(|meta| value.parse(&meta))?;
        }

        Ok(value)
    }

    fn parse(&mut self, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        if meta.path.is_ident("label") {
            let content;
            parenthesized!(content in meta.input);
            self.label = Some((true, content.parse()?));
        }

        Ok(())
    }
}
