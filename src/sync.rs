use std::{ops::Deref, sync::Arc};

use parking_lot::{Mutex, MutexGuard};

pub(crate) trait Dirty {
    fn dirty(&self) -> usize;
    fn increase_dirty(&self);
    fn decrease_dirty(&self);
    fn receiver_promise(&self) -> Vec<UpdatePromise>;
}

/// Core trait for reactive components.
///
/// Objects implementing this trait can react to changes and trigger reactions
/// in dependent components.
#[allow(private_bounds)]
pub trait Reactive: Dirty {
    /// Update the signal and trigger its reaction.
    fn react(&self) -> Vec<UpdatePromise>;

    /// Clone this object as a trait object
    fn clone_box(&self) -> Box<dyn Reactive + Send + Sync>
    where
        Self: 'static;

    fn promise(&self) -> Vec<UpdatePromise>
    where
        Self: 'static,
    {
        self.increase_dirty();
        let self_ = UpdatePromise::new(self.clone_box());
        let mut receivers: Vec<_> = self.receiver_promise();
        receivers.insert(0, self_);
        receivers
    }
}

pub struct UpdatePromise {
    signal: ReactiveRef,
}

impl UpdatePromise {
    /// Creates a new `UpdatePromise` with the given signal.
    pub fn new(signal: ReactiveRef) -> Self {
        Self { signal }
    }

    pub fn from_signal<T: Send + 'static>(signal: &SyncSignal<T>) -> Self {
        Self {
            signal: signal.clone_box(),
        }
    }

    /// Resolves the promise by calling the `react()` method on the signal.
    pub fn resolve(&self) {
        self.signal.decrease_dirty();
        if self.signal.dirty() == 0 {
            self.signal.react();
        }
    }
}

impl Drop for UpdatePromise {
    fn drop(&mut self) {
        self.resolve();
    }
}

/// A type alias for a reference to a `Reactive` object.
type ReactiveRef = Box<dyn Reactive + Send + Sync>;

/// A type alias for a function that generates a value of type `T`.
type GeneratorFn<T> = dyn Fn(&SyncSignal<T>) -> T + Send + Sync;

/// A reactive signal that can be observed and updated.
///
/// SyncSignal is the foundation for reactive programming in this library.
/// It can:
/// - Hold a value that can be read with `get()` or `borrow()`
/// - Be updated with new values via `send()`
/// - Depend on other signals and react to their changes
/// - Have other signals depend on it
pub struct SyncSignal<T> {
    inner: Arc<Mutex<T>>,
    generator: Option<Arc<Box<GeneratorFn<T>>>>,
    receivers: Arc<Mutex<Vec<ReactiveRef>>>,
    suspended: Arc<Mutex<bool>>,
    dirty: Arc<Mutex<usize>>,
}

impl<T> SyncSignal<T> {
    /// Creates a new independent signal with an initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
            generator: None,
            receivers: Arc::new(Mutex::new(Vec::new())),
            suspended: Arc::new(Mutex::new(false)),
            dirty: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a signal that depends on other signals.
    ///
    /// # Parameters
    ///
    /// - `processor`: Function that computes the signal's value from its dependencies
    /// - `effect`: Side effect function called when the signal changes, receives both
    ///   the signal reference and the newly computed value
    ///
    /// # Example
    ///
    /// ```rust
    /// // Create a signal that reacts to changes in another signal
    /// let count = SyncSignal::new(0);
    /// let doubled = SyncSignal::driven(
    ///     || count.get() * 2,
    ///     |_, new_value| println!("Doubled value is now: {}", new_value)
    /// );
    /// count.add_receiver(&doubled); // Register the dependency
    /// ```
    pub fn driven(
        processor: impl Fn() -> T + Send + Sync + 'static,
        effect: impl Fn(&SyncSignal<T>, &T) -> () + Send + Sync + 'static,
    ) -> Self {
        let mut signal = Self {
            inner: Arc::new(Mutex::new(processor())),
            generator: None,
            receivers: Arc::new(Mutex::new(Vec::new())),
            suspended: Arc::new(Mutex::new(false)),
            dirty: Arc::new(Mutex::new(0)),
        };
        signal.generator = Some(Arc::new(Box::new(move |s| {
            let x = processor();
            effect(&s, &x);
            x
        })));
        signal
    }

    /// Gets the current value of the signal (cloned).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.lock().clone()
    }

    /// Gets the current value of the signal by reference.
    ///
    /// This method will not trigger any reactions or computations.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.inner.lock()
    }

    /// Updates the signal value and notifies all dependent signals.
    ///
    /// This triggers the `react()` method on all receivers (dependent signals).
    pub fn send(&self, value: T) -> Vec<UpdatePromise> {
        *self.inner.lock() = value;
        if *self.suspended.lock() {
            return Vec::new();
        }
        self.receivers
            .lock()
            .iter()
            .flat_map(|receiver| receiver.promise())
            .collect()
    }

    /// Registers a dependent signal that will react when this signal changes.
    pub fn add_receiver<U: Clone>(&self, dependent: impl Deref<Target = U>)
    where
        U: Into<ReactiveRef>,
    {
        self.receivers.lock().push(((*dependent).clone()).into());
    }

    /// Temporarily prevents this signal from notifying its dependents when changed.
    ///
    /// This is useful to avoid unnecessary reactions during batch updates.
    pub fn suspend(&self) {
        *self.suspended.lock() = true;
    }

    /// Re-enables notifications to dependent signals after suspension.
    pub fn resume(&self) {
        *self.suspended.lock() = false;
    }

    /// Creates a new independent signal with the same value and receivers.
    pub fn deep_clone(&self) -> Self
    where
        T: Clone,
    {
        Self {
            inner: Arc::new(Mutex::new(self.inner.lock().clone())),
            generator: self.generator.clone(),
            receivers: Arc::new(Mutex::new(
                self.receivers
                    .lock()
                    .iter()
                    .map(|receiver| receiver.clone_box())
                    .collect(),
            )),
            suspended: Arc::new(Mutex::new(*self.suspended.lock())),
            dirty: Arc::new(Mutex::new(*self.dirty.lock())),
        }
    }
}

impl<T: Send> Reactive for SyncSignal<T> {
    fn react(&self) -> Vec<UpdatePromise> {
        let generator = self.generator.as_ref().unwrap();
        let value = (generator.as_ref())(self);
        self.send(value)
    }

    fn clone_box(&self) -> Box<dyn Reactive + Send + Sync>
    where
        Self: 'static,
    {
        Box::new(self.clone())
    }
}

impl<T> Clone for SyncSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            generator: self.generator.clone(),
            receivers: self.receivers.clone(),
            suspended: self.suspended.clone(),
            dirty: self.dirty.clone(),
        }
    }
}

impl<T> Dirty for SyncSignal<T> {
    fn dirty(&self) -> usize {
        *self.dirty.lock()
    }

    fn increase_dirty(&self) {
        *self.dirty.lock() += 1;
    }

    fn decrease_dirty(&self) {
        *self.dirty.lock() -= 1;
    }

    fn receiver_promise(&self) -> Vec<UpdatePromise> {
        self.receivers
            .lock()
            .iter()
            .flat_map(|receiver| receiver.promise())
            .collect()
    }
}

impl<T: Reactive + Send + Sync + 'static> From<T> for Box<dyn Reactive + Send + Sync> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl<T: Reactive + Send + 'static> From<T> for Box<dyn Reactive + Send + Sync>
where
    T: Sync,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}
