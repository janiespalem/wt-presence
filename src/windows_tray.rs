use std::{sync::mpsc, thread};

use anyhow::{Context, Result, bail};
use tokio::sync::mpsc::UnboundedSender;
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, PostQuitMessage, PostThreadMessageW, TranslateMessage,
        WM_QUIT,
    },
};

pub struct WindowsTray {
    thread_id: u32,
    worker: Option<thread::JoinHandle<()>>,
}

impl WindowsTray {
    pub fn start(dashboard_url: String, exit: UnboundedSender<()>) -> Result<Self> {
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("wt-presence-tray".to_owned())
            .spawn(move || tray_thread(dashboard_url, exit, ready_sender))
            .context("start Windows tray thread")?;
        let thread_id = ready_receiver
            .recv()
            .context("Windows tray thread stopped during startup")??;
        Ok(Self {
            thread_id,
            worker: Some(worker),
        })
    }
}

impl Drop for WindowsTray {
    fn drop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn tray_thread(
    dashboard_url: String,
    exit: UnboundedSender<()>,
    ready: mpsc::SyncSender<Result<u32, anyhow::Error>>,
) {
    let (tray, thread_id) = match create_tray(dashboard_url, exit) {
        Ok(value) => value,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(thread_id)).is_err() {
        return;
    }

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    drop(tray);
}

fn create_tray(
    dashboard_url: String,
    exit: UnboundedSender<()>,
) -> Result<(tray_icon::TrayIcon, u32)> {
    let menu = Menu::new();
    let open = MenuItem::with_id("open-dashboard", "Open dashboard", true, None);
    let quit = MenuItem::with_id("quit", "Quit WT Presence", true, None);
    menu.append_items(&[&open, &quit])
        .context("build Windows tray menu")?;

    let open_id = open.id().clone();
    let quit_id = quit.id().clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == open_id {
            let _ = webbrowser::open(&dashboard_url);
        } else if event.id == quit_id {
            let _ = exit.send(());
            unsafe { PostQuitMessage(0) };
        }
    }));

    let icon = tray_icon()?;
    let tray = TrayIconBuilder::new()
        .with_tooltip("WT Presence")
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .build()
        .context("create Windows tray icon")?;
    let thread_id = unsafe { GetCurrentThreadId() };
    if thread_id == 0 {
        bail!("Windows did not return a tray thread ID");
    }
    Ok((tray, thread_id))
}

fn tray_icon() -> Result<Icon> {
    const SIZE: u32 = 32;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    for y in 5_u32..27 {
        for x in 5_u32..27 {
            let wing = (y > 13 && y < 19 && x > 4 && x < 28)
                || (x > 13 && x < 19 && y > 4 && y < 28)
                || (x.abs_diff(y) < 2 && x > 9 && x < 23);
            if wing {
                let offset = ((y * SIZE + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&[232, 162, 58, 255]);
            }
        }
    }
    Icon::from_rgba(rgba, SIZE, SIZE).context("create tray icon pixels")
}
