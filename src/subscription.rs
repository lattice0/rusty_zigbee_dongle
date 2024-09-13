#[allow(unused_imports)]
use crate::utils::{error, trace, warn};
use std::collections::VecDeque;

pub struct Predicate<T>(pub Box<dyn Fn(&T) -> bool + Send + Sync>);
pub struct Action<T>(pub Box<dyn FnOnce(&T) + Send + Sync>);
pub struct Event<T>(pub Box<dyn Fn(&T) + Send + Sync>);

#[derive(Debug)]
pub enum Subscription<T> {
    SingleShot(Predicate<T>, Action<T>),
    Event(Predicate<T>, Event<T>),
}

impl<T> Subscription<T> {
    fn is_single_shot(&self) -> bool {
        matches!(self, Subscription::SingleShot(_, _))
    }

    fn into_action(self) -> Option<(Predicate<T>, Action<T>)> {
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

impl<T> std::fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action")
    }
}

impl<T> std::fmt::Debug for Event<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action")
    }
}

pub struct SubscriptionService<T> {
    subscriptions: VecDeque<Subscription<T>>,
}

impl<T: Clone + PartialEq + std::fmt::Debug> Default for SubscriptionService<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq + std::fmt::Debug> SubscriptionService<T> {
    pub fn new() -> Self {
        Self {
            subscriptions: VecDeque::new(),
        }
    }

    pub fn subscribe(&mut self, subscription: Subscription<T>) {
        trace!("adding subscription {:?}", subscription);
        self.subscriptions.push_front(subscription);
    }

    pub fn notify(&mut self, value: T) -> Result<(), SubscriptionError> {
        if let Some((position, is_single_shot)) = self
            .subscriptions
            .iter_mut()
            .enumerate()
            .find(|(_, s)| match s {
                Subscription::SingleShot(predicate, _) => predicate.0(&value),
                Subscription::Event(predicate, _) => predicate.0(&value),
            })
            .map(|x| (x.0, x.1.is_single_shot()))
        {
            if is_single_shot {
                let subscription = self
                    .subscriptions
                    .remove(position)
                    .ok_or(SubscriptionError::MissingSubscription)?;
                let action = subscription
                    .into_action()
                    .ok_or(SubscriptionError::NotAction)?
                    .1;
                action.0(&value);
            } else {
                let subscription = self
                    .subscriptions
                    .get_mut(position)
                    .ok_or(SubscriptionError::MissingSubscription)?;
                match subscription {
                    Subscription::SingleShot(_, _) => return Err(SubscriptionError::Unreachable),
                    Subscription::Event(_, action) => {
                        action.0(&value);
                    }
                }
            }
        } else {
            warn!("No subscription found for {:?}", value);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SubscriptionError {
    MissingSubscription,
    NotAction,
    Unreachable,
    Send,
}
