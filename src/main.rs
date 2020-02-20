//! Backend subscription example
//!
//! `Backend` refers to some hard work in the background.
//! Shall happen in a separate thread/task/whatever and send
//! messages on progress/events to the UI
//!

use iced::{Application, Column, Command, Element, Settings, Subscription};
use iced_futures::futures::channel::mpsc;

use backend::{Backend, BackendMessage};

pub fn main() {
    SubscriptionExample::run(Settings::default())
}

struct SubscriptionExample {
    tx: Option<mpsc::Sender<BackendMessage>>,
    backend: Backend,
}

#[derive(Debug, Clone)]
enum Message {
    BackendMsg(BackendMessage),
}

impl Application for SubscriptionExample {
    type Executor = iced_futures::executor::AsyncStd;
    type Message = Message;

    fn new() -> (SubscriptionExample, Command<Message>) {
        let example = SubscriptionExample {
            tx: None,
            backend: Backend,
        };
        (example, Command::none())
    }

    fn title(&self) -> String {
        String::from("BackendSubscription - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("Message: {:?}", message);
        match message {
            Message::BackendMsg(msg) => match msg {
                BackendMessage::Tx(tx) => {
                    self.tx = Some(tx);
                    self.backend.start_work(self.tx.as_mut().unwrap());
                }
                BackendMessage::Msg1 => {
                    self.backend.continue_work(self.tx.as_mut().unwrap());
                }
                _ => ()
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        println!("Subscription called");
        backend::subscribe().map(Message::BackendMsg)
    }

    fn view(&mut self) -> Element<Message> {
        Column::new().into()
    }
}

mod backend {
    use iced::futures;
    use iced_futures::futures::channel::mpsc;

    #[derive(Debug, Clone)]
    pub enum BackendMessage {
        Tx(mpsc::Sender<BackendMessage>),
        Msg1,
        Msg2,
        Msg3,
    }

    pub struct Backend;

    impl Backend {
        pub fn start_work(&self, tx: &mut mpsc::Sender<BackendMessage>) {
            println!("start something");
            tx.start_send(BackendMessage::Msg1).ok();
        }

        pub fn continue_work(&self, tx: &mut mpsc::Sender<BackendMessage>) {
            println!("continue something");
            tx.start_send(BackendMessage::Msg2).ok();
        }
    }

    pub fn subscribe() -> iced::Subscription<BackendMessage> {
        iced::Subscription::from_recipe(BackendSubscription)
    }

    pub struct BackendSubscription;

    impl<H, I> iced_native::subscription::Recipe<H, I> for BackendSubscription
    where
        H: std::hash::Hasher,
    {
        type Output = BackendMessage;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;
            std::any::TypeId::of::<Self>().hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            let (mut tx, rx) = mpsc::channel(1);
            let msg = BackendMessage::Tx(tx.clone());
            tx.start_send(msg).ok();

            rx.boxed()
        }
    }
}
