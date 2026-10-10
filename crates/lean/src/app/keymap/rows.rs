use gpui_kit::{App, SharedString};

use super::names::{command_title, context_label};
use super::overrides::{Override, overrides};
use super::registry::{Command, Registry};

pub(crate) struct Row {
    pub(crate) id: SharedString,
    pub(crate) title: SharedString,
    pub(crate) context: SharedString,
    pub(crate) keys: Vec<SharedString>,
    pub(crate) customized: bool,
    pub(crate) available: bool,
    scopes: Vec<Option<SharedString>>,
}

pub(crate) fn rows(cx: &App) -> Vec<Row> {
    let overrides = overrides(cx);
    let Some(registry) = cx.try_global::<Registry>() else {
        return Vec::new();
    };
    let mut rows: Vec<Row> = registry
        .commands
        .iter()
        .map(|command| command_row(command, overrides.get(command.id.as_str())))
        .collect();
    rows.extend(
        overrides
            .iter()
            .filter(|(id, _)| registry.command(id).is_none())
            .map(|(id, entry)| unknown_row(id, entry, cx)),
    );
    rows
}

pub(crate) fn conflicts(row: &Row, rows: &[Row]) -> Vec<SharedString> {
    rows.iter()
        .filter(|other| other.id != row.id && other.available && overlaps(row, other))
        .map(|other| other.title.clone())
        .collect()
}

fn overlaps(row: &Row, other: &Row) -> bool {
    row.keys.iter().any(|keys| other.keys.contains(keys))
        && row.scopes.iter().any(|scope| other.scopes.contains(scope))
}

fn command_row(command: &Command, entry: Option<&Override>) -> Row {
    let scopes: Vec<Option<SharedString>> = command
        .contexts()
        .into_iter()
        .map(|context| context.map(|predicate| predicate.to_string().into()))
        .collect();
    let keys = match entry {
        None => command.default_keys(),
        Some(Override::Unbound) => Vec::new(),
        Some(Override::Keys(keys)) => keys.clone(),
    };
    Row {
        id: command.id.clone(),
        title: command.title.clone(),
        context: scope_labels(&scopes),
        keys,
        customized: entry.is_some(),
        available: true,
        scopes,
    }
}

fn unknown_row(id: &str, entry: &Override, cx: &App) -> Row {
    let keys = match entry {
        Override::Unbound => Vec::new(),
        Override::Keys(keys) => keys.clone(),
    };
    let scopes = vec![None];
    Row {
        id: id.to_owned().into(),
        title: command_title(id),
        context: scope_labels(&scopes),
        keys,
        customized: true,
        available: cx.build_action(id, None).is_ok(),
        scopes,
    }
}

fn scope_labels(scopes: &[Option<SharedString>]) -> SharedString {
    let mut labels: Vec<SharedString> = Vec::new();
    for scope in scopes {
        let label = context_label(scope.as_deref());
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    labels.join(", ").into()
}
