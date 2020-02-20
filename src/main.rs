//! Backend subscription example
//!
//! `Backend` refers to some hard work in the background.
//! Shall happen in a separate thread/task/whatever and send
//! messages on progress/events to the UI
//!

use iced::{button, Application, Button, Column, Command, Element, Settings, Subscription, Text};
use iced_futures::futures::channel::mpsc;

pub fn main() {
    SubscriptionExample::run(Settings::default())
}

enum SubscriptionExample {
    Loading,
    Ready {
        backend: mpsc::Sender<backend::Action>,
        button: button::State,
    },
}

#[derive(Debug, Clone)]
enum Message {
    BackendMsg(backend::Message),
    StartWorkA,
}

impl Application for SubscriptionExample {
    type Executor = iced_futures::executor::AsyncStd;
    type Message = Message;

    fn new() -> (SubscriptionExample, Command<Message>) {
        let example = SubscriptionExample::Loading;
        (example, Command::none())
    }

    fn title(&self) -> String {
        String::from("BackendSubscription - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("Message: {:?}", message);
        match message {
            Message::BackendMsg(msg) => match msg {
                backend::Message::Ready(tx) => {
                    *self = SubscriptionExample::Ready {
                        backend: tx,
                        button: button::State::new(),
                    };
                }
                backend::Message::WorkAStarted => { /* show progress in ui */ }
                backend::Message::WorkADone => { /* show result in ui */ }
                backend::Message::WorkBStarted => { /* show progress in ui */ }
                backend::Message::WorkBDone => {
                    /* show progress in ui*/
                    println!("start cleaning up");
                    if let SubscriptionExample::Ready { backend, .. } = self {
                        backend.start_send(backend::Action::Cleanup).unwrap();
                    }
                }
                _ => (),
            },
            Message::StartWorkA => {
                if let SubscriptionExample::Ready { backend, .. } = self {
                    backend.start_send(backend::Action::StartWorkA).unwrap();
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        println!("Subscription called");
        backend::subscribe().map(Message::BackendMsg)
    }

    fn view(&mut self) -> Element<Message> {
        if let SubscriptionExample::Ready { button, .. } = self {
            Column::new()
                .push(Button::new(button, Text::new("start A")).on_press(Message::StartWorkA))
                .into()
        } else {
            Column::new().into()
        }
    }
}

mod backend {
    use iced::futures;
    use iced_futures::futures::channel::mpsc;

    #[derive(Debug, Clone)]
    pub enum Message {
        Ready(mpsc::Sender<Action>),
        WorkAStarted,
        WorkADone,
        WorkBStarted,
        WorkBDone,
        CleanUpStarted,
        CleanUpFailed,
    }

    #[derive(Debug, Clone)]
    pub enum Action {
        StartWorkA,
        StartWorkB,
        Cleanup,
    }

    pub fn subscribe() -> iced::Subscription<Message> {
        iced::Subscription::from_recipe(BackendSubscription)
    }

    pub struct BackendSubscription;

    impl<H, I> iced_native::subscription::Recipe<H, I> for BackendSubscription
    where
        H: std::hash::Hasher,
    {
        type Output = Message;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;
            std::any::TypeId::of::<Self>().hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            let (tx, rx) = mpsc::channel(1);

            futures::stream::once(async { Message::Ready(tx) })
                .chain(rx.map(|action| {
                    match action {
                        Action::StartWorkA => {
                            /* how to actually start work,
                            immediately return `Message::WorkAStarted`?
                            and the result of `A` later? */
                            Message::WorkAStarted
                        }
                        Action::StartWorkB => Message::WorkBStarted,
                        Action::Cleanup => Message::CleanUpStarted,
                    }
                }))
                .boxed()
        }
    }
}
