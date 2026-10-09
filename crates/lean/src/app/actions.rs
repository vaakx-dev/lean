use std::path::PathBuf;

use gpui_kit::actions;

actions!(
    lean,
    [
        About,
        ClearRecentProjects,
        CloseActiveEditor,
        CollapseAllFolders,
        NextEditor,
        OpenFolder,
        OpenRecent,
        PreviousEditor,
        Quit,
        RefreshExplorer,
        SaveActiveEditor,
        ToggleSidebar,
        ToggleTerminal,
    ]
);

#[derive(Clone, PartialEq, Debug, gpui_kit::Action)]
#[action(namespace = lean, no_json)]
pub struct OpenRecentProject {
    pub path: PathBuf,
}

#[derive(Clone, PartialEq, Debug, gpui_kit::Action)]
#[action(namespace = lean, no_json)]
pub struct ShowPanel {
    pub id: &'static str,
}
