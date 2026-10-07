use alacritty_terminal::event::{Event, EventListener};
use futures::channel::mpsc::UnboundedSender;

#[derive(Clone)]
pub(crate) struct EventProxy(UnboundedSender<Event>);

impl EventProxy {
    pub(crate) fn new(sender: UnboundedSender<Event>) -> Self {
        Self(sender)
    }
}

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        if let Err(error) = self.0.unbounded_send(event) {
            log::debug!("terminal event dropped: {error}");
        }
    }
}
