mod dbus;
mod localize;

use cosmic::{
    applet::menu_button,
    iced::{Alignment, Length, Task, core::time, task, window::Id},
    prelude::*,
    widget::{column, container, icon, mouse_area, row, space, text},
};
use dbus::*;
use std::sync::{Arc, Mutex};

struct Applet {
    core: cosmic::Core,
    is_active: bool,
    timer_handle: Option<task::Handle>,
    lock_state: LockState,
    dbus_connection: Arc<Mutex<Option<zbus::Connection>>>,
    times: Option<usize>,
    popup_id: Option<Id>,
}

#[derive(Debug, Clone)]
enum Message {
    LockSwitch,
    PopupClosed(Id),
    PopupToggle,
    Locked,
    AppError,
    LockTime(Option<usize>),
}

#[derive(Debug, Clone)]
enum LockState {
    Unlocked,
    Acquiring,
    Locked,
}

impl PartialEq for LockState {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (LockState::Unlocked, LockState::Unlocked)
                | (LockState::Acquiring, LockState::Acquiring)
                | (LockState::Locked, LockState::Locked)
        )
    }
}

impl cosmic::Application for Applet {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();

    const APP_ID: &'static str = "com.example.caffeine";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        //TODO:需要添加错误处理
        (
            Self {
                core,
                is_active: false,
                timer_handle: None,
                lock_state: LockState::Unlocked,
                dbus_connection: Arc::new(Mutex::new(None)),
                times: None,
                popup_id: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::LockSwitch => {
                match self.lock_state {
                    LockState::Acquiring => {
                        // Do nothing, still acquiring lock
                        Task::none()
                    }
                    LockState::Locked => {
                        self.dbus_connection.lock().unwrap().take();
                        self.lock_state = LockState::Unlocked;
                        self.is_active = false;
                        Task::none()
                    }
                    LockState::Unlocked => {
                        self.lock_state = LockState::Acquiring;
                        let dbus_connection = self.dbus_connection.clone();
                        //需要错误处理
                        Task::perform(
                            async move {
                                let connection = create_dbus_connection().await.unwrap();
                                apply_inhibit(&connection).await.unwrap();
                                *dbus_connection.lock().unwrap() = Some(connection);
                            },
                            |_| cosmic::Action::from(Message::Locked),
                        )
                    }
                }
            }
            Message::AppError => Task::none(),
            Message::PopupClosed(id) => {
                if self.popup_id == Some(id) {
                    self.popup_id = None;
                }
                Task::none()
            }
            Message::PopupToggle => {
                if let Some(id) = self.popup_id {
                    cosmic::surface::surface_task(cosmic::surface::action::destroy_popup(id))
                } else {
                    cosmic::surface::surface_task(cosmic::surface::action::app_popup(
                        |_| Default::default(),
                        |applet: &mut Applet| {
                            let new_id = Id::unique();
                            applet.popup_id.replace(new_id);
                            applet.core.applet.get_popup_settings(
                                applet.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            )
                        },
                        None,
                    ))
                }
            }
            Message::Locked => {
                self.lock_state = LockState::Locked;
                self.is_active = true;
                Task::none()
            }
            Message::LockTime(minutes) => {
                if let Some(handle) = self.timer_handle.take() {
                    handle.abort();
                }
                if minutes == self.times && self.lock_state == LockState::Locked {
                    Task::done(Message::LockSwitch).map(cosmic::Action::from)
                } else {
                    self.times = minutes;
                    match minutes {
                        None => {
                            self.times.take();
                            if self.lock_state == LockState::Unlocked {
                                Task::done(Message::LockSwitch).map(cosmic::Action::from)
                            } else {
                                Task::none()
                            }
                        }
                        Some(mintues) => {
                            self.times = Some(mintues);
                            let (task, handle) = Task::perform(
                                async move {
                                    tokio::time::sleep(time::minutes(mintues as u64)).await;
                                },
                                |_| Message::LockSwitch,
                            )
                            .map(cosmic::Action::from)
                            .abortable();
                            self.timer_handle = Some(handle);
                            if self.lock_state == LockState::Locked {
                                task
                            } else {
                                Task::done(Message::LockSwitch)
                                    .map(cosmic::Action::from)
                                    .chain(task)
                            }
                        }
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let icon_name = if self.is_active {
            "caffeine-cup-full"
        } else {
            "caffeine-cup-empty"
        };

        mouse_area(
            self.core
                .applet
                .icon_button(icon_name)
                .on_press(Message::LockSwitch),
        )
        .on_right_press(Message::PopupToggle)
        .into()
    }

    fn view_window(&self, id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        if matches!(self.popup_id, Some(p) if p == id) {
            let time_options = [
                (Some(15), fl!("Fifteen-minutes")),
                (Some(30), fl!("Thirty-minutes")),
                (Some(60), fl!("One-hour")),
                (None, fl!("Infinite")),
            ];
            let menu_buttons = column(
                time_options
                    .into_iter()
                    .map(|(minutes, label)| {
                        let is_selected = self.times == minutes && self.is_active;
                        menu_button(
                            row![
                                column![text::body(label)].width(Length::Fill),
                                if is_selected {
                                    container(
                                        icon::from_name("emblem-ok-symbolic")
                                            .size(12)
                                            .symbolic(true),
                                    )
                                } else {
                                    container(space::horizontal().width(12.0))
                                }
                            ]
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::LockTime(minutes))
                        .into()
                    })
                    .collect::<Vec<Element<_>>>(),
            );
            self.core.applet.popup_container(menu_buttons).into()
        } else {
            text("").into()
        }
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: cosmic::iced::window::Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }
}

pub fn run() -> cosmic::iced::Result {
    localize::localize();
    cosmic::applet::run::<Applet>(())
}
