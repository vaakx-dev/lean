use gpui_kit::SharedString;

const GLOBAL_CONTEXT: &str = "Global";

pub(super) fn command_title(action_name: &str) -> SharedString {
    let name = action_name.rsplit("::").next().unwrap_or(action_name);
    sentence_case(name).into()
}

pub(super) fn context_label(context: Option<&str>) -> SharedString {
    let Some(context) = context else {
        return GLOBAL_CONTEXT.into();
    };
    let name: String = context
        .chars()
        .skip_while(|character| !character.is_alphanumeric())
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect();
    if name.is_empty() {
        return context.to_owned().into();
    }
    let mut label = sentence_case(&name);
    if let Some(first) = label.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    label.into()
}

fn sentence_case(name: &str) -> String {
    let mut words = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            words.push(' ');
            words.extend(character.to_lowercase());
        } else {
            words.push(character);
        }
    }
    words
}
