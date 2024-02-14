#![allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]

use anyhow::{Context, Result};
use line_index::{LineIndex, TextRange};
use log::{as_debug, as_display, debug, error, warn};
use lsp_types::{
    DeleteFilesParams, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentSymbolParams,
    DocumentSymbolResponse, FileOperationFilter, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, MarkupContent, MarkupKind, OneOf,
    PositionEncodingKind, Registration, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo,
    TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkDoneProgressOptions, WorkspaceFileOperationsServerCapabilities,
    WorkspaceServerCapabilities,
};
use ropey::Rope;

use self::index::Index;
use crate::{
    state::{self, FileBuilder},
    GlobalState,
};

mod compile;
mod document_symbols;
mod hover;
pub mod index;
mod semantic_tokens;

pub fn initialize(
    state: &mut GlobalState<'_>,
    params: InitializeParams,
) -> Result<InitializeResult> {
    log::trace!("{params:#?}");

    // Select the most preferred position encoding defined by the client.
    state.encoding = params
        .capabilities
        .general
        .as_ref()
        .and_then(|general| general.position_encodings.as_ref())
        .and_then(|encodings| encodings.first())
        .cloned()
        .unwrap_or(PositionEncodingKind::UTF16);

    if let Some(projects) = params
        .root_uri
        .and_then(|root| mabo_project::discover(root.path()).ok())
    {
        for path in projects
            .into_iter()
            .inspect(|project| debug!(path = as_debug!(project.project_path); "found project"))
            .flat_map(|project| project.files)
        {
            let Ok(text) = std::fs::read_to_string(&path) else {
                error!(path = as_debug!(path); "failed reading file content");
                continue;
            };

            let Ok(uri) = Url::from_file_path(&path) else {
                error!(path = as_debug!(path); "failed parsing file path as URI");
                continue;
            };

            state
                .files
                .insert(uri.clone(), create_file(&state.encoding, uri, text));
        }
    }

    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: env!("CARGO_CRATE_NAME").to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        }),
        capabilities: ServerCapabilities {
            position_encoding: Some(state.encoding.clone()),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: None,
                file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                    did_delete: Some(FileOperationRegistrationOptions {
                        filters: vec![FileOperationFilter {
                            scheme: Some("file".to_owned()),
                            pattern: FileOperationPattern {
                                glob: "**/*.mabo".to_owned(),
                                matches: Some(FileOperationPatternKind::File),
                                options: None,
                            },
                        }],
                    }),
                    ..WorkspaceFileOperationsServerCapabilities::default()
                }),
            }),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(false),
                    },
                    legend: SemanticTokensLegend {
                        token_types: semantic_tokens::TOKEN_TYPES.to_vec(),
                        token_modifiers: semantic_tokens::TOKEN_MODIFIERS.to_vec(),
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
    let file = if let Some(file) = state
        .files
        .get(&params.text_document.uri)
        .filter(|file| file.borrow_content() == &text)
    {
        file
    } else {
        debug!("file missing from state");

        let file = create_file(&state.encoding, params.text_document.uri.clone(), text);

        state.files.insert(params.text_document.uri.clone(), file);
        &state.files[&params.text_document.uri]
    };

    if let Err(e) = state.client.publish_diagnostics(
        params.text_document.uri,
        file.borrow_schema()
            .as_ref()
            .err()
            .map(|diag| vec![diag.clone()])
            .unwrap_or_default(),
        None,
    ) {
        error!(error = as_debug!(e); "failed publishing diagnostics");
    }
}

pub fn did_change(state: &mut GlobalState<'_>, mut params: DidChangeTextDocumentParams) {
    fn is_full(changes: &[TextDocumentContentChangeEvent]) -> bool {
        changes.len() == 1 && changes.first().is_some_and(|change| change.range.is_none())
    }

    debug!(
        uri = as_display!(params.text_document.uri),
        full = as_display!(is_full(&params.content_changes));
        "schema changed",
    );

    let file = if is_full(&params.content_changes) {
        let text = params.content_changes.remove(0).text;
        create_file(&state.encoding, params.text_document.uri.clone(), text)
    } else {
        let Some(file) = state.files.remove(&params.text_document.uri) else {
            warn!("missing state for changed file");
            return;
        };

        update_file(params.text_document.uri.clone(), file, |rope, index| {
            for change in params.content_changes {
                let range = match convert_range(index, change.range) {
                    Ok(range) => range,
                    Err(e) => {
                        error!(error = as_debug!(e); "invalid change");
                        continue;
                    }
                };

                let start = rope.byte_to_char(range.start().into());
                let end = rope.byte_to_char(range.end().into());
                rope.remove(start..end);
                rope.insert(start, &change.text);
            }
        })
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

pub fn did_close(_state: &mut GlobalState<'_>, params: DidCloseTextDocumentParams) {
    debug!(uri = as_display!(params.text_document.uri); "schema closed");
}

pub fn did_delete(state: &mut GlobalState<'_>, params: DeleteFilesParams) {
    debug!(files = as_debug!(params.files); "files deleted");
    state
        .files
        .retain(|uri, _| !params.files.iter().any(|file| file.uri == uri.as_str()));
}

pub fn hover(state: &mut GlobalState<'_>, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    debug!(uri = as_display!(uri); "requested hover info");

    Ok(
        if let Some((schema, index)) = state.files.get_mut(&uri).and_then(|file| {
            file.borrow_simplified()
                .as_ref()
                .ok()
                .zip(Some(file.borrow_index()))
        }) {
            hover::visit_schema(index, schema, position)?.map(|(value, range)| Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: Some(range),
            })
        } else {
            None
        },
    )
}

pub fn document_symbol(
    state: &mut GlobalState<'_>,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    debug!(uri = as_display!(params.text_document.uri); "requested document symbols");

    Ok(
        if let Some((schema, index)) = state.files.get(&params.text_document.uri).and_then(|file| {
            file.borrow_schema()
                .as_ref()
                .ok()
                .zip(Some(file.borrow_index()))
        }) {
            Some(document_symbols::visit_schema(index, schema)?.into())
        } else {
            None
        },
    )
}

pub fn semantic_tokens_full(
    state: &mut GlobalState<'_>,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    debug!(uri = as_display!(params.text_document.uri); "requested semantic tokens");

    Ok(
        if let Some((schema, index)) = state.files.get(&params.text_document.uri).and_then(|file| {
            file.borrow_schema()
                .as_ref()
                .ok()
                .zip(Some(file.borrow_index()))
        }) {
            Some(
                SemanticTokens {
                    result_id: None,
                    data: semantic_tokens::Visitor::new(index).visit_schema(schema)?,
                }
                .into(),
            )
        } else {
            None
        },
    )
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

fn convert_range(index: &Index, range: Option<lsp_types::Range>) -> Result<TextRange> {
    let range = range.context("incremental change misses range")?;

    let start = index
        .get_offset(range.start)
        .context("failed to convert start position to byte offset")?;

    let end = index
        .get_offset(range.end)
        .context("failed to convert end position to byte offset")?;

    Ok(TextRange::new(start.try_into()?, end.try_into()?))
}

fn create_file(encoding: &PositionEncodingKind, uri: Url, text: String) -> state::File {
    FileBuilder {
        rope: Rope::from_str(&text),
        index: Index::new(LineIndex::new(&text), encoding),
        content: text,
        schema_builder: |index, schema| compile::compile(uri, schema, index),
        simplified_builder: compile::simplify,
    }
    .build()
}

fn update_file(uri: Url, file: state::File, update: impl FnOnce(&mut Rope, &Index)) -> state::File {
    let mut heads = file.into_heads();

    update(&mut heads.rope, &heads.index);
    let text = String::from(&heads.rope);

    heads.index.update(LineIndex::new(&text));

    FileBuilder {
        rope: heads.rope,
        index: heads.index,
        content: text,
        schema_builder: |index, schema| compile::compile(uri, schema, index),
        simplified_builder: compile::simplify,
    }
    .build()
}
