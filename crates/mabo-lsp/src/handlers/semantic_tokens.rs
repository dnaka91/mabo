use anyhow::{ensure, Result};
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType};
use mabo_parser::{
    Comment, Const, DataType, Definition, Enum, Fields, Generics, Id, Literal, LiteralValue,
    Module, NamedField, Schema, Span, Spanned, Struct, Type, TypeAlias, UnnamedField, Variant,
};

pub(crate) use self::{modifiers::TOKEN_MODIFIERS, types::TOKEN_TYPES};
use super::index::Index;

macro_rules! define_semantic_token_types {
    (
        standard {
            $($standard:ident),* $(,)?
        }

        custom {
            $(($custom:ident, $string:literal)),* $(,)?
        }
    ) => {
        mod types {
            use lsp_types::SemanticTokenType;

            $(pub(super) const $standard: SemanticTokenType = SemanticTokenType::$standard;)*
            $(pub(super) const $custom: SemanticTokenType = SemanticTokenType::new($string);)*

            pub(crate) const TOKEN_TYPES: &[SemanticTokenType] = &[
                $(SemanticTokenType::$standard,)*
                $($custom,)*
            ];
        }
    };
}

define_semantic_token_types! {
    standard {
        NAMESPACE,
        TYPE,
        ENUM,
        STRUCT,
        TYPE_PARAMETER,
        VARIABLE,
        PROPERTY,
        ENUM_MEMBER,
        // KEYWORD,
        COMMENT,
        // STRING,
        NUMBER,
        // OPERATOR,
        // DECORATOR,
    }

    custom {
        (BOOLEAN, "boolean"),
        (BUILTIN_TYPE, "builtinType"),
        (IDENTIFIER, "identifier"),
        (TYPE_ALIAS, "typeAlias"),
    }
}

macro_rules! define_semantic_token_modifiers {
    (
        standard {
            $($standard:ident),* $(,)?
        }
        custom {
            $(($custom:ident, $string:literal)),* $(,)?
        }
    ) => {
        mod modifiers {
            use lsp_types::SemanticTokenModifier;

            $(pub(super) const $standard: SemanticTokenModifier = SemanticTokenModifier::$standard;)*
            $(pub(super) const $custom: SemanticTokenModifier = SemanticTokenModifier::new($string);)*

            pub(crate) const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
                $(SemanticTokenModifier::$standard,)*
                $($custom,)*
            ];
        }
    };
}

define_semantic_token_modifiers! {
    standard {
        DECLARATION,
        STATIC,
        DOCUMENTATION,
    }

    custom {
        (CONSTANT, "constant"),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::expect_used)]
fn token_type_pos(ty: &SemanticTokenType) -> u32 {
    // This should never fail as the above macros ensure every possible constant is in the global
    // list as well. However, if passing a `SemanticTokenType` directly from the `lsp-types` crate
    // it can fail.
    types::TOKEN_TYPES
        .iter()
        .position(|tt| tt == ty)
        .expect("token type missing from global list") as u32
}

fn token_modifier_bitset(modifiers: &[SemanticTokenModifier]) -> u32 {
    modifiers::TOKEN_MODIFIERS
        .iter()
        .enumerate()
        .filter_map(|(i, tm)| modifiers.contains(tm).then_some(i))
        .fold(0, |acc, modifier| acc + (1 << modifier))
}

pub struct Visitor<'a> {
    index: &'a Index,
    tokens: Vec<SemanticToken>,
    delta: lsp_types::Position,
}

impl<'a> Visitor<'a> {
    pub fn new(index: &'a Index) -> Self {
        Self {
            index,
            tokens: Vec::new(),
            delta: lsp_types::Position {
                line: 0,
                character: 0,
            },
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_range(&self, span: Span) -> Result<lsp_types::Range> {
        let range = self.index.get_range(span)?;

        ensure!(
            range.start.line == range.end.line,
            "encountered a multi-line span"
        );

        Ok(range)
    }

    fn lsl(&self, start: lsp_types::Position, end: lsp_types::Position) -> (u32, u32, u32) {
        (
            start.line - self.delta.line,
            start.character
                - if self.delta.line == start.line {
                    self.delta.character
                } else {
                    0
                },
            end.character - start.character,
        )
    }

    fn add_span(
        &mut self,
        span: &impl Spanned,
        token_type: &SemanticTokenType,
        token_modifiers: &[SemanticTokenModifier],
    ) -> Result<()> {
        let range = self.get_range(span.span())?;
        let (delta_line, delta_start, length) = self.lsl(range.start, range.end);

        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: token_type_pos(token_type),
            token_modifiers_bitset: token_modifier_bitset(token_modifiers),
        });
        self.delta = lsp_types::Position {
            line: range.start.line,
            character: range.start.character,
        };

        Ok(())
    }

    pub fn visit_schema(mut self, item: &Schema<'_>) -> Result<Vec<SemanticToken>> {
        for def in &item.definitions {
            self.visit_definition(def)?;
        }

        Ok(self.tokens)
    }

    fn visit_comment(&mut self, item: &Comment<'_>) -> Result<()> {
        for line in &item.0 {
            self.add_span(line, &types::COMMENT, &[modifiers::DOCUMENTATION])?;
        }

        Ok(())
    }

    fn visit_definition(&mut self, item: &Definition<'_>) -> Result<()> {
        match item {
            Definition::Module(m) => self.visit_module(m),
            Definition::Struct(s) => self.visit_struct(s),
            Definition::Enum(e) => self.visit_enum(e),
            Definition::TypeAlias(a) => self.visit_alias(a),
            Definition::Const(c) => self.visit_const(c),
            Definition::Import(_i) => Ok(()),
        }
    }

    fn visit_module(&mut self, item: &Module<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(&item.name, &types::NAMESPACE, &[modifiers::DECLARATION])?;

        for def in &item.definitions {
            self.visit_definition(def)?;
        }

        Ok(())
    }

    fn visit_struct(&mut self, item: &Struct<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(&item.name, &types::STRUCT, &[modifiers::DECLARATION])?;
        self.visit_generics(&item.generics)?;
        self.visit_fields(&item.fields)
    }

    fn visit_enum(&mut self, item: &Enum<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(&item.name, &types::ENUM, &[modifiers::DECLARATION])?;
        self.visit_generics(&item.generics)?;

        for variant in &item.variants {
            self.visit_variant(variant)?;
        }

        Ok(())
    }

    fn visit_variant(&mut self, item: &Variant<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(&item.name, &types::ENUM_MEMBER, &[modifiers::DECLARATION])?;
        self.visit_fields(&item.fields)?;
        self.visit_id(&item.id)
    }

    fn visit_fields(&mut self, item: &Fields<'_>) -> Result<()> {
        match item {
            Fields::Named(named) => {
                for field in named {
                    self.visit_named_field(field)?;
                }
            }
            Fields::Unnamed(unnamed) => {
                for field in unnamed {
                    self.visit_unnamed_field(field)?;
                }
            }
            Fields::Unit => {}
        }

        Ok(())
    }

    fn visit_named_field(&mut self, item: &NamedField<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(&item.name, &types::PROPERTY, &[modifiers::DECLARATION])?;
        self.visit_type(&item.ty)?;
        self.visit_id(&item.id)
    }

    fn visit_unnamed_field(&mut self, item: &UnnamedField<'_>) -> Result<()> {
        self.visit_type(&item.ty)?;
        self.visit_id(&item.id)
    }

    fn visit_id(&mut self, item: &Option<Id>) -> Result<()> {
        if let Some(id) = item {
            self.add_span(id, &types::IDENTIFIER, &[])?;
        }

        Ok(())
    }

    fn visit_alias(&mut self, item: &TypeAlias<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(&item.name, &types::TYPE_ALIAS, &[modifiers::DECLARATION])?;
        self.visit_generics(&item.generics)?;
        self.visit_type(&item.target)
    }

    fn visit_const(&mut self, item: &Const<'_>) -> Result<()> {
        self.visit_comment(&item.comment)?;
        self.add_span(
            &item.name,
            &types::VARIABLE,
            &[
                modifiers::DECLARATION,
                modifiers::STATIC,
                modifiers::CONSTANT,
            ],
        )?;
        self.visit_type(&item.ty)?;
        self.visit_literal(&item.value)
    }

    fn visit_type(&mut self, item: &Type<'_>) -> Result<()> {
        self.add_span(
            item,
            &if matches!(item.value, DataType::External(_)) {
                types::TYPE
            } else {
                types::BUILTIN_TYPE
            },
            &[],
        )
    }

    fn visit_literal(&mut self, item: &Literal) -> Result<()> {
        let token_type = match item.value {
            LiteralValue::Bool(_) => types::BOOLEAN,
            LiteralValue::Int(_) | LiteralValue::Float(_) => types::NUMBER,
            LiteralValue::String(_) | LiteralValue::Bytes(_) => {
                // these can be multiline and need special handling
                return Ok(());
            }
        };

        self.add_span(item, &token_type, &[])
    }

    fn visit_generics(&mut self, item: &Generics<'_>) -> Result<()> {
        for generic in &item.0 {
            self.add_span(generic, &types::TYPE_PARAMETER, &[modifiers::DECLARATION])?;
        }

        Ok(())
    }
}
