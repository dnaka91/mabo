#![allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]

use anyhow::{Context, Result};
use line_index::{LineIndex, TextRange};
use log::{as_debug, as_display, debug, error, warn};
use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
    PositionEncodingKind, Registration, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};
use ropey::Rope;

use crate::{compile, state::FileBuilder, GlobalState};

pub fn initialize(
    _state: &mut GlobalState<'_>,
    _params: InitializeParams,
) -> Result<InitializeResult> {
    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: env!("CARGO_CRATE_NAME").to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        }),
        capabilities: ServerCapabilities {
            position_encoding: Some(PositionEncodingKind::UTF16),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(false),
                    },
                    legend: SemanticTokensLegend {
                        token_types: vec![
                            SemanticTokenType::NAMESPACE,
                            SemanticTokenType::TYPE,
                            SemanticTokenType::CLASS,
                            SemanticTokenType::ENUM,
                            SemanticTokenType::INTERFACE,
                            SemanticTokenType::STRUCT,
                            SemanticTokenType::TYPE_PARAMETER,
                            SemanticTokenType::PARAMETER,
                            SemanticTokenType::VARIABLE,
                            SemanticTokenType::PROPERTY,
                            SemanticTokenType::ENUM_MEMBER,
                            SemanticTokenType::EVENT,
                            SemanticTokenType::FUNCTION,
                            SemanticTokenType::METHOD,
                            SemanticTokenType::MACRO,
                            SemanticTokenType::KEYWORD,
                            SemanticTokenType::MODIFIER,
                            SemanticTokenType::COMMENT,
                            SemanticTokenType::STRING,
                            SemanticTokenType::NUMBER,
                            SemanticTokenType::REGEXP,
                            SemanticTokenType::OPERATOR,
                            SemanticTokenType::DECORATOR,
                        ],
                        token_modifiers: vec![
                            SemanticTokenModifier::DECLARATION,
                            SemanticTokenModifier::DEFINITION,
                            SemanticTokenModifier::READONLY,
                            SemanticTokenModifier::STATIC,
                            SemanticTokenModifier::DEPRECATED,
                            SemanticTokenModifier::ABSTRACT,
                            SemanticTokenModifier::ASYNC,
                            SemanticTokenModifier::MODIFICATION,
                            SemanticTokenModifier::DOCUMENTATION,
                            SemanticTokenModifier::DEFAULT_LIBRARY,
                        ],
                    },
                    range: Some(false),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                }),
            ),
            ..ServerCapabilities::default()
        },
        offset_encoding: None,
    })
}

pub fn initialized(state: &mut GlobalState<'_>, _params: InitializedParams) {
    if let Err(e) = state.reload_settings() {
        error!(error = as_debug!(e); "failed loading initial settings");
    }

    if let Err(e) = state.client.register_capability(vec![Registration {
        id: "1".to_owned(),
        method: "workspace/didChangeConfiguration".to_owned(),
        register_options: None,
    }]) {
        error!(error = as_debug!(e); "failed registering for configuration changes");
    }

    debug!("initialized");
}

pub fn did_open(state: &mut GlobalState<'_>, params: DidOpenTextDocumentParams) {
    debug!(uri = as_display!(params.text_document.uri); "schema opened");

    let text = params.text_document.text;
    let file = FileBuilder {
        rope: Rope::from_str(&text),
        index: LineIndex::new(&text),
        content: text,
        schema_builder: |index, schema| {
            compile::compile(params.text_document.uri.clone(), schema, index)
        },
    }
    .build();

    if let Err(e) = state.client.publish_diagnostics(
        params.text_document.uri.clone(),
        file.borrow_schema()
            .as_ref()
            .err()
            .map(|diag| vec![diag.clone()])
            .unwrap_or_default(),
        None,
    ) {
        error!(error = as_debug!(e); "failed publishing diagnostics");
    }

    state.files.insert(params.text_document.uri, file);
}

pub fn did_change(state: &mut GlobalState<'_>, mut params: DidChangeTextDocumentParams) {
    debug!(uri = as_display!(params.text_document.uri); "schema changed");

    let file = if params.content_changes.len() == 1
        && params
            .content_changes
            .first()
            .is_some_and(|change| change.range.is_none())
    {
        let text = params.content_changes.remove(0).text;
        FileBuilder {
            rope: Rope::from_str(&text),
            index: LineIndex::new(&text),
            content: text,
            schema_builder: |index, schema| {
                compile::compile(params.text_document.uri.clone(), schema, index)
            },
        }
        .build()
    } else {
        let Some(file) = state.files.remove(&params.text_document.uri) else {
            warn!("missing state for changed file");
            return;
        };

        let mut heads = file.into_heads();

        for change in params.content_changes {
            let range = match convert_range(&heads.index, change.range) {
                Ok(range) => range,
                Err(e) => {
                    error!(error = as_debug!(e); "invalid change");
                    continue;
                }
            };

            let start = heads.rope.byte_to_char(range.start().into());
            let end = heads.rope.byte_to_char(range.end().into());
            heads.rope.remove(start..end);
            heads.rope.insert(start, &change.text);
        }

        let text = String::from(&heads.rope);

        FileBuilder {
            rope: heads.rope,
            index: LineIndex::new(&text),
            content: text,
            schema_builder: |index, schema| {
                compile::compile(params.text_document.uri.clone(), schema, index)
            },
        }
        .build()
    };

    if let Err(e) = state.client.publish_diagnostics(
        params.text_document.uri.clone(),
        file.borrow_schema()
            .as_ref()
            .err()
            .map(|diag| vec![diag.clone()])
            .unwrap_or_default(),
        None,
    ) {
        error!(error = as_debug!(e); "failed publishing diagnostics");
    }

    state.files.insert(params.text_document.uri, file);
}

pub fn did_close(state: &mut GlobalState<'_>, params: DidCloseTextDocumentParams) {
    debug!(uri = as_display!(params.text_document.uri); "schema closed");
    state.files.remove(&params.text_document.uri);
}

pub fn semantic_tokens_full(
    _state: &mut GlobalState<'_>,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    debug!(uri = as_display!(params.text_document.uri); "requested semantic tokens");
    Ok(None)
}

pub fn did_change_configuration(
    state: &mut GlobalState<'_>,
    _params: DidChangeConfigurationParams,
) {
    debug!("configuration changed");

    if let Err(e) = state.reload_settings() {
        error!(error = as_debug!(e); "failed loading changed settings");
    }
}

pub fn convert_range(index: &LineIndex, range: Option<lsp_types::Range>) -> Result<TextRange> {
    let range = range.context("incremental change misses range")?;

    let start = index
        .offset(
            index
                .to_utf8(
                    line_index::WideEncoding::Utf16,
                    line_index::WideLineCol {
                        line: range.start.line,
                        col: range.start.character,
                    },
                )
                .context("failed to convert start position to utf-8")?,
        )
        .context("failed to convert start position to byte offset")?;

    let end = index
        .offset(
            index
                .to_utf8(
                    line_index::WideEncoding::Utf16,
                    line_index::WideLineCol {
                        line: range.end.line,
                        col: range.end.character,
                    },
                )
                .context("failed to convert end position to utf-8")?,
        )
        .context("failed to convert end position to byte offset")?;

    Ok(TextRange::new(start, end))
}
