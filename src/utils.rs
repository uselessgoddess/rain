use {
  concurrent_queue::{ConcurrentQueue, PopError},
  std::sync::Arc,
};

pub struct ArxSender<T>(Arc<ConcurrentQueue<T>>);

impl<T> ArxSender<T> {
  pub fn send(self, val: T) {
    let _ = self.0.force_push(val);
  }
}

pub struct ArxReceiver<'a, T>(&'a mut Option<Arc<ConcurrentQueue<T>>>);

impl<'a, T> ArxReceiver<'a, T> {
  pub fn try_recv(&mut self) -> Result<T, PopError> {
    self.0.as_mut().unwrap().pop().inspect(|_| *self.0 = None)
  }
}

pub struct Arx<T> {
  que: Option<Arc<ConcurrentQueue<T>>>,
}

impl<T> Clone for Arx<T> {
  fn clone(&self) -> Self {
    Self { que: self.que.clone() }
  }
}

impl<T> Arx<T> {
  pub fn new() -> Self {
    Self { que: None }
  }

  pub fn ready(&mut self) -> Option<ArxReceiver<T>> {
    if self.que.is_some() { Some(ArxReceiver(&mut self.que)) } else { None }
  }

  pub fn task(&mut self) -> ArxSender<T> {
    let que = Arc::new(ConcurrentQueue::bounded(1));
    self.que = Some(que.clone());
    ArxSender(que)
  }
}

impl<T> Default for Arx<T> {
  fn default() -> Self {
    Self::new()
  }
}
