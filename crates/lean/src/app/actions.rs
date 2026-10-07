use gpui_kit::actions;

actions!(
    lean,
    [
        About,
        CloseActiveEditor,
        CollapseAllFolders,
        NextEditor,
        OpenFolder,
        PreviousEditor,
        Quit,
        RefreshExplorer,
        SaveActiveEditor,
        ShowExplorer,
        ShowSearch,
        ShowSourceControl,
        ToggleSidebar,
        ToggleTerminal,
    ]
);
