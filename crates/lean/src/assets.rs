use std::borrow::Cow;

use anyhow::Result;
use gpui_kit::assets::icon_assets;
use gpui_kit::{AssetSource, SharedString};

pub(crate) struct Assets;

pub(crate) const BUNDLED_THEME: &str = include_str!("../../../assets/themes/lean-dark.json");

const LOGO_PATH: &str = "images/lean.png";
const LOGO: &[u8] = include_bytes!("../../../assets/images/lean.png");

icon_assets!(
    ExtraIcons,
    [
        Braces,
        Clock,
        Code,
        Columns2,
        Eraser,
        FastForward,
        FileCode,
        FileDiff,
        FileImage,
        FileInput,
        FilePlus,
        FileX,
        Files,
        FolderPlus,
        FolderTree,
        GitBranch,
        Image,
        Keyboard,
        List,
        ListCollapse,
        ListTree,
        Lock,
        NotebookText,
        Pencil,
        Puzzle,
        Regex,
        RotateCcw,
        Trash,
        WholeWord,
        Wrench,
        X,
        ZoomIn,
        ZoomOut,
    ]
);

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == LOGO_PATH {
            return Ok(Some(Cow::Borrowed(LOGO)));
        }
        if let Some(icon) = ExtraIcons.load(path)? {
            return Ok(Some(icon));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = ExtraIcons.list(path)?;
        paths.extend(gpui_kit::assets::Assets.list(path)?);
        if LOGO_PATH.starts_with(path) {
            paths.push(LOGO_PATH.into());
        }
        Ok(paths)
    }
}
