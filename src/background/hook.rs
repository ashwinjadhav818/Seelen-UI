use std::{sync::Arc, time::Duration};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN,
            EVENT_OBJECT_CREATE, EVENT_OBJECT_FOCUS, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
            MSG,
        },
    },
};
use winvd::{listen_desktop_events, DesktopEvent};

use crate::{
    apps_config::{AppExtraFlag, SETTINGS_BY_APP}, error_handler::{log_if_error, Result}, seelen::{Seelen, SEELEN}, seelen_weg::SeelenWeg, utils::constants::IGNORE_FOCUS, windows_api::WindowsApi
};

lazy_static! {
    pub static ref HOOK_MANAGER: Arc<Mutex<HookManager>> = Arc::new(Mutex::new(HookManager::new()));
}

pub struct HookManager {
    paused: bool,
    waiting_event: Option<u32>,
    waiting_hwnd: Option<HWND>,
    resume_cb: Option<Box<dyn Fn(&mut HookManager) + Send>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            paused: false,
            waiting_event: None,
            waiting_hwnd: None,
            resume_cb: None,
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
        if let Some(cb) = self.resume_cb.take() {
            cb(self);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause_and_resume_after(&mut self, event: u32, hwnd: HWND) {
        self.pause();
        self.waiting_event = Some(event);
        self.waiting_hwnd = Some(hwnd);
    }

    pub fn set_resume_callback<F>(&mut self, cb: F)
    where
        F: Fn(&mut HookManager) + Send + 'static,
    {
        self.resume_cb = Some(Box::new(cb));
    }

    pub fn is_waiting_for(&self, event: u32, hwnd: HWND) -> bool {
        self.waiting_event == Some(event) && self.waiting_hwnd == Some(hwnd)
    }

    pub fn emit_fake_win_event(&mut self, event: u32, hwnd: HWND) {
        std::thread::spawn(move || {
            win_event_hook(HWINEVENTHOOK::default(), event, hwnd, 0, 0, 0, 0);
        });
    }
}

pub fn process_vd_event(event: DesktopEvent) -> Result<()> {
    let mut seelen = SEELEN.lock();
    for monitor in seelen.monitors_mut() {
        if let Some(wm) = monitor.wm_mut() {
            log_if_error(wm.process_vd_event(&event));
        }
    }

    match event {
        DesktopEvent::WindowChanged(hwnd) => {
            if WindowsApi::is_window(hwnd) {
                if let Some(config) = SETTINGS_BY_APP.lock().get_by_window(hwnd) {
                    if config.options_contains(AppExtraFlag::Pinned)
                        && !winvd::is_pinned_window(hwnd)?
                    {
                        winvd::pin_window(hwnd)?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

impl Seelen {
    pub fn process_win_event(&mut self, event: u32, hwnd: HWND) -> Result<()> {
        match event {
            EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
                // ensure that the taskbar is always hidden
                if self.state().is_weg_enabled() && "Shell_TrayWnd" == WindowsApi::get_class(hwnd)?
                {
                    SeelenWeg::hide_taskbar(true);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub extern "system" fn win_event_hook(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if id_object != 0 {
        return;
    }

    let mut hook_manager = HOOK_MANAGER.lock();
    if hook_manager.is_paused() {
        if hook_manager.is_waiting_for(event, hwnd) {
            hook_manager.resume();
        }
        return;
    }

    /* if event == EVENT_OBJECT_LOCATIONCHANGE {
        return;
    }

    let winevent = match crate::winevent::WinEvent::try_from(event) {
        Ok(event) => event,
        Err(_) => return,
    };

    println!(
        "{:?} || {} || {} || {}",
        winevent,
        WindowsApi::exe(hwnd).unwrap_or_default(),
        WindowsApi::get_class(hwnd).unwrap_or_default(),
        WindowsApi::get_window_text(hwnd)
    ); */

    let title = WindowsApi::get_window_text(hwnd);
    if (event == EVENT_OBJECT_FOCUS || event == EVENT_SYSTEM_FOREGROUND)
        && IGNORE_FOCUS.contains(&title)
    {
        return;
    }

    let mut seelen = SEELEN.lock();
    log_if_error(seelen.process_win_event(event, hwnd));

    for monitor in seelen.monitors_mut() {
        if let Some(toolbar) = monitor.toolbar_mut() {
            log_if_error(toolbar.process_win_event(event, hwnd));
        }

        if let Some(weg) = monitor.weg_mut() {
            log_if_error(weg.process_win_event(event, hwnd));
        }

        if let Some(wm) = monitor.wm_mut() {
            log_if_error(wm.process_win_event(event, hwnd));
        }
    }
}

pub fn register_win_hook() -> Result<()> {
    std::thread::spawn(move || unsafe {
        SetWinEventHook(EVENT_MIN, EVENT_MAX, None, Some(win_event_hook), 0, 0, 0);

        let mut msg: MSG = MSG::default();
        loop {
            if !GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                log::info!("windows event processing shutdown");
                break;
            };
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
            std::thread::sleep(Duration::from_millis(10));
        }
    });

    std::thread::spawn(move || -> Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel::<DesktopEvent>();
        let _notifications_thread = listen_desktop_events(sender)?;
        for event in receiver {
            log_if_error(process_vd_event(event))
        }
        Ok(())
    });
    Ok(())
}
