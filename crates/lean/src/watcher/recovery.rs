use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use gpui_kit::{AsyncApp, Task};

use super::filter::RawEvent;
use super::health::Health;

const FIRST_RETRY: Duration = Duration::from_secs(5);
const LAST_RETRY: Duration = Duration::from_secs(300);
const MAX_DOUBLINGS: u32 = 6;

#[derive(Default)]
pub(super) struct Recovery {
    attempts: u32,
    failing: bool,
    retry: Option<Task<()>>,
}

impl Recovery {
    pub(super) fn retry_started(&mut self) {
        self.retry = None;
    }

    pub(super) fn check(
        &mut self,
        health: &Health,
        requests: &UnboundedSender<RawEvent>,
        cx: &AsyncApp,
    ) -> Option<String> {
        let Some(failure) = health.failure() else {
            if self.failing {
                log::info!("the file watcher recovered");
            }
            *self = Self::default();
            return None;
        };
        if self.retry.is_some() {
            return None;
        }
        self.retry = Some(schedule_retry(self.delay(), requests.clone(), cx));
        self.attempts += 1;
        let first_failure = !self.failing;
        self.failing = true;
        first_failure.then_some(failure)
    }

    fn delay(&self) -> Duration {
        FIRST_RETRY
            .saturating_mul(1 << self.attempts.min(MAX_DOUBLINGS))
            .min(LAST_RETRY)
    }
}

fn schedule_retry(delay: Duration, requests: UnboundedSender<RawEvent>, cx: &AsyncApp) -> Task<()> {
    cx.spawn(async move |cx: &mut AsyncApp| {
        cx.background_executor().timer(delay).await;
        requests.unbounded_send(RawEvent::retry()).ok();
    })
}
