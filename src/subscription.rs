use crate::utils::{err, log};
use std::collections::VecDeque;

pub struct Predicate<T>(pub Box<dyn Fn(&T) -> bool + Send + Sync>);

#[derive(Debug)]
pub enum Subscription<T> {
    SingleShot(Predicate<T>, futures::channel::oneshot::Sender<T>),
    Periodic(Predicate<T>, futures::channel::mpsc::Sender<T>),
}

impl<T> Subscription<T> {
    fn is_single_shot(&self) -> bool {
        matches!(self, Subscription::SingleShot(_, _))
    }

    fn to_single_shot(self) -> Option<(Predicate<T>, futures::channel::oneshot::Sender<T>)> {
        match self {
            Subscription::SingleShot(predicate, tx) => Some((predicate, tx)),
            _ => None,
        }
    }
}

impl<T> std::fmt::Debug for Predicate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Predicate")
    }
}

pub struct SubscriptionService<T> {
    subscriptions: VecDeque<Subscription<T>>,
}

impl<T: Clone + PartialEq + std::fmt::Debug> SubscriptionService<T> {
    pub fn new() -> Self {
        Self {
            subscriptions: VecDeque::new(),
        }
    }

    pub fn subscribe(&mut self, subscription: Subscription<T>) {
        log!("adding subscription {:?}", subscription);
        self.subscriptions.push_front(subscription);
    }

    pub fn notify(&mut self, value: T) -> Result<(), SubscriptionError> {
        if let Some((position, is_single_shot)) = self
            .subscriptions
            .iter_mut()
            .enumerate()
            .find(|(_, s)| match s {
                Subscription::SingleShot(predicate, _) => predicate.0(&value),
                Subscription::Periodic(predicate, _) => predicate.0(&value),
            })
            .map(|x| (x.0, x.1.is_single_shot()))
        {
            if is_single_shot {
                println!("found single shot subscription");
                let subscription = self
                    .subscriptions
                    .remove(position)
                    .ok_or(SubscriptionError::MissingSubscription)?;
                println!("subscription: {:?}", subscription);
                println!("subscriptions: {:?}", self.subscriptions);
                let tx = subscription
                    .to_single_shot()
                    .ok_or(SubscriptionError::NotSingleShot)?
                    .1;
                tx.send(value.clone())
                    .map_err(|_| SubscriptionError::Send)?;
                log!("sent packet");
            } else {
                println!("found multiple shot subscription");
                let subscription = self.subscriptions.get_mut(position).unwrap();
                match subscription {
                    Subscription::SingleShot(_, _) => return Err(SubscriptionError::Unreachable),
                    Subscription::Periodic(_, tx) => {
                        tx.try_send(value.clone())
                            .map_err(|_| SubscriptionError::Send)?;
                    }
                }
            }
        } else {
            err!("No subscription found for {:?}", value);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SubscriptionError {
    MissingSubscription,
    NotSingleShot,
    Unreachable,
    Send,
}
