use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct TerminalLaunch {
    pub(crate) environment: HashMap<String, String>,
    pub(crate) startup_command: Option<String>,
}
