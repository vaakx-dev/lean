use gpui_kit::App;

pub(crate) struct Snapshot {
    pub(crate) encode: Box<dyn FnOnce() -> Vec<u8> + Send>,
    pub(crate) on_saved: Option<Box<dyn FnOnce(&mut App)>>,
}

impl Snapshot {
    pub(crate) fn new(encode: impl FnOnce() -> Vec<u8> + Send + 'static) -> Self {
        Self {
            encode: Box::new(encode),
            on_saved: None,
        }
    }

    pub(crate) fn on_saved(mut self, on_saved: impl FnOnce(&mut App) + 'static) -> Self {
        self.on_saved = Some(Box::new(on_saved));
        self
    }
}
