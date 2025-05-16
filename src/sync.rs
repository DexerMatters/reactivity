use crate::api::{Receptive, SealedSignalTrait};
use parking_lot::RwLock;
use std::sync::Arc;

/// A thread-safe reactive signal that can be observed and updated.
///
/// `Signal` is the thread-safe implementation for reactive programming
/// in multi-threaded contexts. It uses `Arc` and `RwLock` internally,
/// making it safe to share across thread boundaries.
///
/// For single-threaded signals, use `reactivity::Signal` instead.
///
/// # Usage
///
/// ```rust
/// use std::thread;
/// use reactivity::sync::Signal;
/// use reactivity::signal;
///
/// // Create a thread-safe signal
/// let count = signal!(0);
///
/// // Create a derived signal
/// let doubled = signal!([count] count * 2);
///
/// // Clone for use in another thread
/// let count_clone = count.clone();
///
/// // Update the signal from another thread
/// thread::spawn(move || {
///     count_clone.send(5);
/// }).join().unwrap();
///
/// // The change propagates automatically
/// assert_eq!(doubled.get(), 10);
/// ```
///
/// # When to use
///
/// Use `sync::Signal` when signals need to be shared across multiple threads.
/// If all signals will be accessed from the same thread, use `reactivity::Signal`
/// instead for better performance.
#[derive(Clone)]
pub struct Signal<T> {
    /// The current value of the signal
    inner: Arc<RwLock<T>>,
    /// Optional effect function called when the signal is updated
    effect: Option<Arc<dyn Fn(&Signal<T>, &T) + Send + Sync>>,
    /// Optional function that computes the signal's value
    processor: Option<Arc<dyn Fn() -> T + Send + Sync>>,
    /// List of receivers that depend on this signal
    receivers: Arc<RwLock<Vec<Box<dyn Receptive + Send + Sync>>>>,
    /// Counter tracking pending updates
    dirty: Arc<RwLock<usize>>,
}

impl<T: 'static> Signal<T> {
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
    /// use std::thread;
    /// use reactivity::sync::Signal;
    ///
    /// // Create a signal that reacts to changes in another signal
    /// let count = Signal::new(0);
    /// let doubled = Signal::driven(
    ///     || count.get() * 2,
    ///     |_, new_value| println!("Doubled value is now: {}", new_value)
    /// );
    ///
    /// // Add doubled as a receiver of count (no Box::new needed)
    /// count.add_receiver(doubled.clone());
    ///
    /// // Update from another thread
    /// let count_clone = count.clone();
    /// thread::spawn(move || {
    ///     count_clone.send(5);
    /// }).join().unwrap();
    /// ```
    pub fn driven<F>(processor: F, effect: impl Fn(&Signal<T>, &T) + Send + Sync + 'static) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self::init(
            Arc::new(RwLock::new(processor())),
            Some(Arc::new(effect)),
            Some(Arc::new(processor)),
            Arc::new(RwLock::new(Vec::new())),
            Arc::new(RwLock::new(0)),
        )
    }
}

impl<T: 'static> SealedSignalTrait for Signal<T> {
    type Inner = T;
    type Rc<U: ?Sized> = Arc<U>;
    type Ptr<U> = RwLock<U>;
    type Effect = dyn Fn(&Signal<T>, &T) + Send + Sync;
    type Processor = dyn Fn() -> T + Send + Sync;
    type Receiver = dyn Receptive + Send + Sync;

    fn init(
        inner: Arc<RwLock<Self::Inner>>,
        effect: Option<Arc<Self::Effect>>,
        processor: Option<Arc<Self::Processor>>,
        receivers: Arc<RwLock<Vec<Box<Self::Receiver>>>>,
        dirty: Arc<RwLock<usize>>,
    ) -> Self {
        Self {
            inner,
            effect,
            processor,
            receivers,
            dirty,
        }
    }

    fn inner(&self) -> &Arc<RwLock<T>> {
        &self.inner
    }

    fn effect(&self) -> Option<&Arc<Self::Effect>> {
        self.effect.as_ref()
    }

    fn processor(&self) -> Option<&Arc<Self::Processor>> {
        self.processor.as_ref()
    }

    fn receivers(&self) -> &Arc<RwLock<Vec<Box<Self::Receiver>>>> {
        &self.receivers
    }

    fn dirty(&self) -> &Arc<RwLock<usize>> {
        &self.dirty
    }
}
