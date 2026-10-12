use roxmltree::{Document, Node, ParsingOptions};

const FULL: &str = "100%";
const PERCENT: &str = "%";

pub(super) fn aspect_ratio(text: &str) -> Option<f32> {
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(text, options).ok()?;
    let svg = document.root_element();
    let (width, width_unit) = length(svg.attribute("width").unwrap_or(FULL))?;
    let (height, height_unit) = length(svg.attribute("height").unwrap_or(FULL))?;
    if width_unit != height_unit {
        return None;
    }
    let ratio = height / width;
    let ratio = if width_unit == PERCENT {
        ratio * view_box_ratio(svg)?
    } else {
        ratio
    };
    (ratio.is_finite() && ratio > 0.).then_some(ratio)
}

fn length(value: &str) -> Option<(f32, &str)> {
    let value = value.trim();
    let unit_start = value
        .find(|c: char| c.is_ascii_alphabetic() || c == '%')
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(unit_start);
    let unit = if unit == "px" { "" } else { unit };
    Some((number.trim().parse().ok()?, unit))
}

fn view_box_ratio(svg: Node<'_, '_>) -> Option<f32> {
    let numbers = svg
        .attribute("viewBox")?
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    match numbers[..] {
        [_, _, width, height] if width > 0. && height > 0. => Some(height / width),
        _ => None,
    }
}
