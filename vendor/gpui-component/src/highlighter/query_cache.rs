use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, PoisonError},
};

use anyhow::{Context as _, Result};
use gpui::SharedString;
use tree_sitter::{Language, Query};

use super::{GrammarConfig, LanguageRegistry};

/// The compiled queries of one language, shared by all of its highlighters.
pub(crate) struct HighlightQueries {
    pub(crate) query: Arc<Query>,
    pub(crate) injections_query: Option<Arc<Query>>,
    pub(crate) locals_pattern_index: usize,
    pub(crate) highlights_pattern_index: usize,
    pub(crate) non_local_variable_patterns: Vec<bool>,
    pub(crate) injection_content_capture_index: Option<u32>,
    pub(crate) injection_language_capture_index: Option<u32>,
    pub(crate) local_scope_capture_index: Option<u32>,
    pub(crate) local_def_capture_index: Option<u32>,
    pub(crate) local_def_value_capture_index: Option<u32>,
    pub(crate) local_ref_capture_index: Option<u32>,
}

/// Compiled queries by canonical language name, stamped with the registry
/// generation they were compiled under so re-registrations recompile.
type QueryCache<T> = LazyLock<Mutex<HashMap<SharedString, (u64, T)>>>;

static HIGHLIGHT_QUERIES: QueryCache<Arc<HighlightQueries>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static INJECTED_HIGHLIGHT_QUERIES: QueryCache<Arc<Query>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn lookup<T: Clone>(cache: &QueryCache<T>, name: &SharedString, generation: u64) -> Option<T> {
    cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(name)
        .filter(|(stamp, _)| *stamp == generation)
        .map(|(_, value)| value.clone())
}

fn store<T>(cache: &QueryCache<T>, name: &SharedString, generation: u64, value: T) {
    cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(name.clone(), (generation, value));
}

/// The host highlight queries for `config`, compiled once per language.
pub(crate) fn highlight_queries(
    config: &GrammarConfig,
    grammar: &Language,
) -> Result<Arc<HighlightQueries>> {
    let generation = LanguageRegistry::singleton().generation();
    if let Some(queries) = lookup(&HIGHLIGHT_QUERIES, &config.name, generation) {
        return Ok(queries);
    }
    let queries = Arc::new(HighlightQueries::build(config, grammar)?);
    store(
        &HIGHLIGHT_QUERIES,
        &config.name,
        generation,
        queries.clone(),
    );
    Ok(queries)
}

/// The highlights query used when `config`'s language is injected into
/// another, compiled once per language.
pub(crate) fn injected_highlight_query(
    language_name: &str,
    config: &GrammarConfig,
) -> Option<Arc<Query>> {
    let generation = LanguageRegistry::singleton().generation();
    if let Some(query) = lookup(&INJECTED_HIGHLIGHT_QUERIES, &config.name, generation) {
        return Some(query);
    }
    let grammar = LanguageRegistry::singleton().grammar(language_name).ok()?;
    let query = match Query::new(&grammar, &config.highlights) {
        Ok(query) => Arc::new(query),
        Err(error) => {
            tracing::error!(
                "failed to build injection query for {:?}: {:?}",
                config.name,
                error
            );
            return None;
        }
    };
    store(
        &INJECTED_HIGHLIGHT_QUERIES,
        &config.name,
        generation,
        query.clone(),
    );
    Some(query)
}

impl HighlightQueries {
    /// https://github.com/tree-sitter/tree-sitter/blob/v0.26.8/crates/highlight/src/highlight.rs#L339
    fn build(config: &GrammarConfig, grammar: &Language) -> Result<Self> {
        // Concatenate the query strings, keeping track of the start offset of each section.
        let mut query_source = String::new();
        query_source.push_str(&config.injections);
        let locals_query_offset = query_source.len();
        query_source.push_str(&config.locals);
        let highlights_query_offset = query_source.len();
        query_source.push_str(&config.highlights);

        // Construct a single query by concatenating the three query strings, but record the
        // range of pattern indices that belong to each individual string.
        let mut query = Query::new(grammar, &query_source).context("new query")?;

        let mut locals_pattern_index = 0;
        let mut highlights_pattern_index = 0;
        for i in 0..(query.pattern_count()) {
            let pattern_offset = query.start_byte_for_pattern(i);
            if pattern_offset < highlights_query_offset {
                highlights_pattern_index += 1;
                if pattern_offset < locals_query_offset {
                    locals_pattern_index += 1;
                }
            }
        }

        let injections_query = if !config.injections.is_empty() {
            Query::new(grammar, &config.injections).ok().map(Arc::new)
        } else {
            None
        };

        // Injection layers are computed separately during parsing, so do not
        // emit injection captures from the main highlight query.
        for pattern_index in 0..locals_pattern_index {
            query.disable_pattern(pattern_index);
        }

        // Find all of the highlighting patterns that are disabled for nodes that
        // have been identified as local variables.
        let non_local_variable_patterns = (0..query.pattern_count())
            .map(|i| {
                query
                    .property_predicates(i)
                    .iter()
                    .any(|(prop, positive)| !*positive && prop.key.as_ref() == "local")
            })
            .collect();

        // Store the numeric ids for all of the special captures.
        let injection_capture_index = |capture: &str| {
            injections_query.as_ref().and_then(|query| {
                query
                    .capture_names()
                    .iter()
                    .position(|name| *name == capture)
                    .map(|i| i as u32)
            })
        };
        let injection_content_capture_index = injection_capture_index("injection.content");
        let injection_language_capture_index = injection_capture_index("injection.language");

        let mut local_def_capture_index = None;
        let mut local_def_value_capture_index = None;
        let mut local_ref_capture_index = None;
        let mut local_scope_capture_index = None;
        for (i, name) in query.capture_names().iter().enumerate() {
            let i = Some(i as u32);
            match *name {
                "local.definition" => local_def_capture_index = i,
                "local.definition-value" => local_def_value_capture_index = i,
                "local.reference" => local_ref_capture_index = i,
                "local.scope" => local_scope_capture_index = i,
                _ => {}
            }
        }

        Ok(Self {
            query: Arc::new(query),
            injections_query,
            locals_pattern_index,
            highlights_pattern_index,
            non_local_variable_patterns,
            injection_content_capture_index,
            injection_language_capture_index,
            local_scope_capture_index,
            local_def_capture_index,
            local_def_value_capture_index,
            local_ref_capture_index,
        })
    }
}
