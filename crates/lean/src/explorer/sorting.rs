use crate::project::DirectoryEntry;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Sorting {
    pub(super) folders_first: bool,
    pub(super) natural: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum NamePart {
    Number(usize, String),
    Text(String),
}

pub(super) fn sort_entries(entries: &mut [DirectoryEntry], sorting: Sorting) {
    entries.sort_by_cached_key(|entry| {
        (
            sorting.folders_first && !entry.is_dir,
            name_key(&entry.name, sorting.natural),
        )
    });
}

fn name_key(name: &str, natural: bool) -> Vec<NamePart> {
    let lowercase = name.to_lowercase();
    if !natural {
        return vec![NamePart::Text(lowercase)];
    }
    let mut parts = Vec::new();
    let mut rest = lowercase.as_str();
    while let Some(first) = rest.chars().next() {
        let is_number = first.is_ascii_digit();
        let end = rest
            .find(|character: char| character.is_ascii_digit() != is_number)
            .unwrap_or(rest.len());
        let (chunk, tail) = rest.split_at(end);
        parts.push(if is_number {
            number_part(chunk)
        } else {
            NamePart::Text(chunk.to_owned())
        });
        rest = tail;
    }
    parts
}

fn number_part(digits: &str) -> NamePart {
    let significant = digits.trim_start_matches('0');
    NamePart::Number(significant.len(), significant.to_owned())
}
