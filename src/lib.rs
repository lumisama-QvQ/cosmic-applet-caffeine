mod dbus;
mod localize;

use std::future::pending;

use cosmic::{
    applet::menu_button,
    iced::{
        Alignment, Length, Subscription, Task,
        core::time,
        futures::{SinkExt, StreamExt},
        stream, task,
        window::Id,
    },
    prelude::*,
    widget::{column, container, icon, mouse_area, row, space, text},
};
use dbus::*;

struct Applet {
    core: cosmic::Core,
    timer_handle: Option<task::Handle>,
    lock_state: LockState,
    dbus_connection: Option<zbus::Connection>,
    times: Option<usize>,
    popup_id: Option<Id>,
}

#[derive(Debug, Clone)]
enum Message {
    LockSwitch,
    PopupClosed(Id),
    Unlock,
    PopupToggle,
    Locked(zbus::Connection),
    AppError(String),
    LockTime(Option<usize>),
}

#[derive(Debug, Clone, PartialEq)]
enum LockState {
    Unlocked,
    Acquiring,
    Locked,
}

impl Applet {
    fn is_locked(&self) -> bool {
        self.lock_state == LockState::Locked
    }

    fn cancel_timer(&mut self) {
        if let Some(handle) = self.timer_handle.take() {
            handle.abort();
        }
    }

    fn unlock(&mut self) {
        self.cancel_timer();
        self.lock_state = LockState::Unlocked;
        self.times = None;
        self.dbus_connection.take();
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
        (
            Self {
                core,
                timer_handle: None,
                lock_state: LockState::Unlocked,
                dbus_connection: None,
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
                        self.unlock();
                        Task::none()
                    }
                    LockState::Unlocked => {
                        self.lock_state = LockState::Acquiring;
                        Task::perform(
                            async move {
                                let connection = create_dbus_connection().await?;
                                apply_inhibit(&connection).await?;
                                Ok::<zbus::Connection, anyhow::Error>(connection)
                            },
                            |res| match res {
                                Ok(conn) => Message::Locked(conn).into(),
                                Err(err) => Message::AppError(err.to_string()).into(),
                            },
                        )
                    }
                }
            }
            Message::Unlock => {
                self.unlock();
                Task::none()
            }
            Message::AppError(error) => {
                self.unlock();
                eprintln!("Caffeine Applet Error: {error}");
                let _ = notify_rust::Notification::new()
                    .summary("Caffeine Applet Error")
                    .body(&error)
                    .show_async();
                Task::none()
            }
            Message::PopupClosed(id) => {
                if self.popup_id == Some(id) {
                    self.popup_id = None;
                }
                Task::none()
            }
            Message::PopupToggle => {
                if let Some(id) = self.popup_id.take() {
                    cosmic::surface::surface_task(cosmic::surface::action::destroy_popup(id))
                } else {
                    cosmic::surface::surface_task(cosmic::surface::action::app_popup(
                        |_| Default::default(),
                        |applet: &mut Applet| {
                            let new_id = Id::unique();
                            applet.popup_id.replace(new_id);
                            applet.core.applet.get_popup_settings(
                                applet
                                    .core
                                    .main_window_id()
                                    .expect("Main window should exist"),
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
            Message::Locked(conn) => {
                self.dbus_connection = Some(conn);
                self.lock_state = LockState::Locked;
                Task::none()
            }
            Message::LockTime(minutes) => {
                if minutes == self.times && self.is_locked() {
                    self.unlock();
                    Task::none()
                } else {
                    self.cancel_timer();
                    self.times = minutes;
                    match minutes {
                        None => {
                            if self.lock_state == LockState::Unlocked {
                                Task::done(Message::LockSwitch.into())
                            } else {
                                Task::none()
                            }
                        }
                        Some(min) => {
                            let (task, handle) = Task::perform(
                                async move {
                                    tokio::time::sleep(time::minutes(min as u64)).await;
                                },
                                |_| Message::LockSwitch.into(),
                            )
                            .abortable();
                            self.timer_handle = Some(handle);
                            if self.lock_state == LockState::Locked {
                                task
                            } else {
                                Task::done(Message::LockSwitch.into()).chain(task)
                            }
                        }
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let icon_name = if self.is_locked() {
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
                        let is_selected = self.times == minutes && self.is_locked();
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

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        Subscription::run(|| {
            type Sender = cosmic::iced::futures::channel::mpsc::Sender<Message>;
            stream::channel::<Message>(100, move |mut output: Sender| async move {
                let manager = match async {
                    let conn = zbus::Connection::system().await?;
                    let manager = logind_zbus::manager::ManagerProxy::new(&conn).await?;
                    Ok::<_, anyhow::Error>(manager)
                }
                .await
                {
                    Ok(res) => res,
                    Err(err) => {
                        let _ = output.send(Message::AppError(err.to_string())).await;
                        let _ = pending::<()>().await;
                        return;
                    }
                };
                if let Ok(mut signal_stream) = manager.receive_prepare_for_sleep().await {
                    while let Some(signal) = signal_stream.next().await {
                        if let Ok(args) = signal.args()
                            && !args.start()
                        {
                            let _ = output.send(Message::Unlock).await;
                        }
                    }
                }
            })
        })
    }
}

pub fn run() -> cosmic::iced::Result {
    localize::localize();
    cosmic::applet::run::<Applet>(())
}
