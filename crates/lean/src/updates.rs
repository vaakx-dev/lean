mod archive;
mod build;
mod github;
mod install;
mod notices;
pub(crate) mod settings;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use gpui_kit::{App, Global, Task};

use crate::app::actions::CheckForUpdates;
use build::Build;
use github::Release;
use settings::{CHANNEL, CHECK_AUTOMATICALLY};

pub(crate) use build::version_label;

const FIRST_CHECK_DELAY: Duration = Duration::from_secs(10);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Clone, Copy, PartialEq)]
enum Trigger {
    Automatic,
    Manual,
}

#[derive(Clone)]
struct Installed {
    label: String,
    program: PathBuf,
}

#[derive(Default)]
struct Updater {
    busy: bool,
    offered: Option<String>,
    installed: Option<Installed>,
}

impl Global for Updater {}

pub(crate) fn init(cx: &mut App) {
    cx.set_global(Updater::default());
    cx.on_action(|_: &CheckForUpdates, cx| check(Trigger::Manual, cx));
    if build::current().is_none() {
        return;
    }
    check_periodically(cx).detach();
    CHANNEL
        .observe_app(cx, |_, cx| check(Trigger::Automatic, cx))
        .detach();
}

fn check_periodically(cx: &mut App) -> Task<()> {
    cx.spawn(async move |cx| {
        let mut delay = FIRST_CHECK_DELAY;
        loop {
            cx.background_executor().timer(delay).await;
            cx.update(|cx| {
                if CHECK_AUTOMATICALLY.get(cx) {
                    check(Trigger::Automatic, cx);
                }
            });
            delay = CHECK_INTERVAL;
        }
    })
}

fn check(trigger: Trigger, cx: &mut App) {
    let Some(build) = build::current() else {
        if trigger == Trigger::Manual {
            notices::show(notices::not_updatable(), cx);
        }
        return;
    };
    let updater = cx.global::<Updater>();
    if let Some(installed) = updater.installed.clone() {
        if trigger == Trigger::Manual {
            notices::show(notices::installed(installed), cx);
        }
        return;
    }
    if updater.busy {
        return;
    }
    cx.global_mut::<Updater>().busy = true;
    let channel = CHANNEL.get(cx);
    cx.spawn(async move |cx| {
        let latest = cx
            .background_executor()
            .spawn(async move { github::latest(channel) })
            .await;
        cx.update(|cx| {
            cx.global_mut::<Updater>().busy = false;
            report(latest, build, trigger, cx);
        });
    })
    .detach();
}

fn report(latest: Result<Release>, build: Build, trigger: Trigger, cx: &mut App) {
    match latest {
        Ok(release) if build.is_superseded_by(&release) => offer(release, trigger, cx),
        Ok(_) if trigger == Trigger::Manual => notices::show(notices::up_to_date(), cx),
        Ok(_) => {}
        Err(error) => {
            log::warn!("update check failed: {error:#}");
            if trigger == Trigger::Manual {
                notices::show(notices::failed(&error), cx);
            }
        }
    }
}

fn offer(release: Release, trigger: Trigger, cx: &mut App) {
    let label = release.label();
    let updater = cx.global_mut::<Updater>();
    if trigger == Trigger::Automatic && updater.offered.as_ref() == Some(&label) {
        return;
    }
    updater.offered = Some(label);
    notices::show(notices::available(release), cx);
}

fn start_install(release: Release, cx: &mut App) {
    let updater = cx.global_mut::<Updater>();
    if updater.busy {
        return;
    }
    updater.busy = true;
    let label = release.label();
    notices::show(notices::downloading(&label), cx);
    cx.spawn(async move |cx| {
        let program = cx
            .background_executor()
            .spawn(async move { install::run(&release) })
            .await;
        cx.update(|cx| finish_install(program.map(|program| Installed { label, program }), cx));
    })
    .detach();
}

fn finish_install(installed: Result<Installed>, cx: &mut App) {
    let updater = cx.global_mut::<Updater>();
    updater.busy = false;
    match installed {
        Ok(installed) => {
            updater.installed = Some(installed.clone());
            notices::show(notices::installed(installed), cx);
        }
        Err(error) => {
            log::error!("update failed: {error:#}");
            notices::show(notices::failed(&error), cx);
        }
    }
}
