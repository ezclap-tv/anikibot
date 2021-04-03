use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::mpsc;

pub struct Handle<T> {
    sender: Arc<mpsc::Sender<T>>,
    value: Option<T>,
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        // the pool is bounded, so if we are holding one of its values, it won't be
        // full, which means try_send won't fail
        let value = self
            .value
            .take()
            .expect("Pool Handle value was None when it should've been Some");
        if self.sender.try_send(value).is_err() {
            panic!("Pool receiver was full")
        }
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value
            .as_ref()
            .expect("Pool Handle value was None when it should've been Some")
    }
}

impl<T> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
            .as_mut()
            .expect("Pool Handle value was None when it should've been Some")
    }
}

pub struct Pool<T> {
    sender: Arc<mpsc::Sender<T>>,
    receiver: mpsc::Receiver<T>,
}

impl<T> Pool<T> {
    pub fn new(size: usize, mut init: impl FnMut() -> T) -> Pool<T> {
        let (sender, receiver) = mpsc::channel(size);
        for _ in 0..size {
            if sender.try_send(init()).is_err() {
                panic!("Pool was full when it should've been empty")
            }
        }
        Pool {
            sender: Arc::new(sender),
            receiver,
        }
    }

    pub async fn get(&mut self) -> Handle<T> {
        Handle {
            sender: self.sender.clone(),
            value: Some(
                self.receiver
                    .recv()
                    .await
                    .expect("Pool channel was closed unexpectedly"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn pool_with_1_item() {
        let mut pool = Pool::<usize>::new(1, || 0usize);

        tokio::spawn({
            let mut value = pool.get().await;
            async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                *value += 1usize;
                drop(value);
            }
        });
        assert_eq!(*(pool.get().await), 1usize);
    }
}
