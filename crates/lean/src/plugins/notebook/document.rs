use anyhow::{Result, bail};
use serde::Serialize as _;
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Map, Value, json};
use uuid::Uuid;

const DEFAULT_LANGUAGE: &str = "python";
const NBFORMAT_INDENT: &[u8] = b" ";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellKind {
    Code,
    Markdown,
    Raw,
}

impl CellKind {
    fn from_name(name: &str) -> Option<CellKind> {
        match name {
            "code" => Some(CellKind::Code),
            "markdown" => Some(CellKind::Markdown),
            "raw" => Some(CellKind::Raw),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            CellKind::Code => "code",
            CellKind::Markdown => "markdown",
            CellKind::Raw => "raw",
        }
    }
}

pub(crate) struct CellData {
    pub(crate) kind: CellKind,
    pub(crate) source: String,
    pub(crate) outputs: Vec<Value>,
    pub(crate) execution_count: Option<u64>,
    pub(crate) fields: Map<String, Value>,
    pub(crate) verbatim: Option<Value>,
}

impl CellData {
    pub(crate) fn new(kind: CellKind) -> CellData {
        let mut fields = Map::new();
        fields.insert(
            "id".into(),
            Value::String(Uuid::new_v4().simple().to_string()),
        );
        fields.insert("metadata".into(), json!({}));
        CellData {
            kind,
            source: String::new(),
            outputs: Vec::new(),
            execution_count: None,
            fields,
            verbatim: None,
        }
    }

    fn unknown(value: Value) -> CellData {
        CellData {
            kind: CellKind::Raw,
            source: value.get("source").map(join_text).unwrap_or_default(),
            outputs: Vec::new(),
            execution_count: None,
            fields: Map::new(),
            verbatim: Some(value),
        }
    }
}

pub(crate) struct Document {
    pub(crate) cells: Vec<CellData>,
    pub(crate) fields: Map<String, Value>,
}

impl Document {
    pub(crate) fn parse(content: &str) -> Result<Document> {
        if content.trim().is_empty() {
            return Ok(Document::empty());
        }
        let Value::Object(mut fields) = serde_json::from_str(content)? else {
            bail!("the file is not a Jupyter notebook");
        };
        let Some(Value::Array(cells)) = fields.remove("cells") else {
            bail!("the notebook has no cell list (only nbformat 4 is supported)");
        };
        Ok(Document {
            cells: cells.into_iter().map(parse_cell).collect(),
            fields,
        })
    }

    fn empty() -> Document {
        let mut fields = Map::new();
        fields.insert(
            "metadata".into(),
            json!({
                "kernelspec": {
                    "display_name": "Python 3",
                    "language": "python",
                    "name": "python3"
                },
                "language_info": { "name": "python" }
            }),
        );
        fields.insert("nbformat".into(), json!(4));
        fields.insert("nbformat_minor".into(), json!(5));
        Document {
            cells: vec![CellData::new(CellKind::Code)],
            fields,
        }
    }

    pub(crate) fn language(&self) -> String {
        let metadata = self.fields.get("metadata");
        metadata
            .and_then(|metadata| metadata.pointer("/kernelspec/language"))
            .or_else(|| metadata.and_then(|metadata| metadata.pointer("/language_info/name")))
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_LANGUAGE)
            .to_lowercase()
    }

    pub(crate) fn serialize(mut self) -> Result<String> {
        let cells = self.cells.into_iter().map(cell_json).collect();
        self.fields.insert("cells".into(), Value::Array(cells));
        let mut document = Value::Object(self.fields);
        document.sort_all_objects();
        let mut bytes = Vec::new();
        let formatter = PrettyFormatter::with_indent(NBFORMAT_INDENT);
        document.serialize(&mut Serializer::with_formatter(&mut bytes, formatter))?;
        let mut content = String::from_utf8(bytes)?;
        content.push('\n');
        Ok(content)
    }
}

fn parse_cell(value: Value) -> CellData {
    let kind = value
        .get("cell_type")
        .and_then(Value::as_str)
        .and_then(CellKind::from_name);
    match (kind, value) {
        (Some(kind), Value::Object(fields)) => known_cell(kind, fields),
        (_, value) => CellData::unknown(value),
    }
}

fn known_cell(kind: CellKind, mut fields: Map<String, Value>) -> CellData {
    fields.remove("cell_type");
    let source = fields
        .remove("source")
        .map(|source| join_text(&source))
        .unwrap_or_default();
    let (outputs, execution_count) = if kind == CellKind::Code {
        code_results(&mut fields)
    } else {
        (Vec::new(), None)
    };
    CellData {
        kind,
        source,
        outputs,
        execution_count,
        fields,
        verbatim: None,
    }
}

fn code_results(fields: &mut Map<String, Value>) -> (Vec<Value>, Option<u64>) {
    let outputs = match fields.remove("outputs") {
        Some(Value::Array(outputs)) => outputs,
        _ => Vec::new(),
    };
    let execution_count = fields
        .remove("execution_count")
        .and_then(|count| count.as_u64());
    (outputs, execution_count)
}

fn cell_json(cell: CellData) -> Value {
    if let Some(verbatim) = cell.verbatim {
        return verbatim;
    }
    let mut fields = cell.fields;
    fields.insert("cell_type".into(), Value::String(cell.kind.name().into()));
    fields.insert("source".into(), split_lines(&cell.source));
    fields.entry("metadata").or_insert_with(|| json!({}));
    if cell.kind == CellKind::Code {
        fields.insert("outputs".into(), Value::Array(cell.outputs));
        fields.insert(
            "execution_count".into(),
            cell.execution_count.map_or(Value::Null, Value::from),
        );
    }
    Value::Object(fields)
}

pub(crate) fn join_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(lines) => lines.iter().filter_map(Value::as_str).collect(),
        _ => String::new(),
    }
}

fn split_lines(text: &str) -> Value {
    Value::Array(
        text.split_inclusive('\n')
            .map(|line| Value::String(line.into()))
            .collect(),
    )
}
