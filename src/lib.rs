use api::{Receptive, SealedSignalTrait};

use std::{cell::RefCell, rc::Rc};

pub mod api;
pub mod sync;

/// A reactive signal that can be observed and updated.
///
/// `Signal` is the standard implementation for reactive programming
/// in single-threaded contexts. It uses `Rc` and `RefCell` internally.
/// For thread-safe signals, use `reactivity::sync::Signal` instead.
///
/// # Usage
///
/// ```rust
/// use reactivity::Signal;
/// use reactivity::signal;
///
/// // Create a basic signal
/// let count = signal!(0);
///
/// // Create a derived signal
/// let doubled = signal!([count] count * 2);
///
/// // Manually establish dependency (the signal! macro does this automatically)
/// count.add_receiver(doubled);
///
/// // Update the original signal
/// count.send(5);
///
/// // The change propagates automatically
/// assert_eq!(doubled.get(), 10);
/// ```
///
/// # When to use
///
/// Use `Signal` when all signals will be accessed from the same thread.
/// If you need to share signals across multiple threads, use `sync::Signal` instead.
#[derive(Clone)]
pub struct Signal<T> {
    /// The current value of the signal
    inner: Rc<RefCell<T>>,
    /// Optional effect function called when the signal is updated
    effect: Option<Rc<dyn Fn(&Signal<T>, &T)>>,
    /// Optional function that computes the signal's value
    processor: Option<Rc<dyn Fn() -> T>>,
    /// List of receivers that depend on this signal
    receivers: Rc<RefCell<Vec<Box<dyn Receptive>>>>,
    /// Counter tracking pending updates
    dirty: Rc<RefCell<usize>>,
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
    /// // Create a signal that reacts to changes in another signal
    /// let count = Signal::new(0);
    /// let doubled = Signal::driven(
    ///     || count.get() * 2,
    ///     |_, new_value| println!("Doubled value is now: {}", new_value)
    /// );
    /// count.add_receiver(Box::new(doubled));
    /// ```
    pub fn driven<F>(processor: F, effect: impl Fn(&Signal<T>, &T) + 'static) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self::init(
            Rc::new(RefCell::new(processor())),
            Some(Rc::new(effect)),
            Some(Rc::new(processor)),
            Rc::new(RefCell::new(Vec::new())),
            Rc::new(RefCell::new(0)),
        )
    }
}

impl<T: 'static> SealedSignalTrait for Signal<T> {
    type Inner = T;
    type Rc<U: ?Sized> = Rc<U>;
    type Ptr<U> = RefCell<U>;
    type Effect = dyn Fn(&Signal<T>, &T);
    type Processor = dyn Fn() -> T;
    type Receiver = dyn Receptive;

    fn init(
        inner: Rc<RefCell<Self::Inner>>,
        effect: Option<Rc<Self::Effect>>,
        processor: Option<Rc<Self::Processor>>,
        receivers: Rc<RefCell<Vec<Box<Self::Receiver>>>>,
        dirty: Rc<RefCell<usize>>,
    ) -> Self {
        Self {
            inner,
            effect,
            processor,
            receivers,
            dirty,
        }
    }

    fn inner(&self) -> &Rc<RefCell<T>> {
        &self.inner
    }

    fn effect(&self) -> Option<&Rc<Self::Effect>> {
        self.effect.as_ref()
    }

    fn processor(&self) -> Option<&Rc<Self::Processor>> {
        self.processor.as_ref()
    }

    fn receivers(&self) -> &Rc<RefCell<Vec<Box<Self::Receiver>>>> {
        &self.receivers
    }

    fn dirty(&self) -> &Rc<RefCell<usize>> {
        &self.dirty
    }
}

/// A reactive signal that can be observed and updated.
/// It is thread-safe and can be used in concurrent environments.
///
/// It can:
/// - Hold a value that can be read with `get()` or `borrow()`
/// - Be updated with new values via `send()`
/// - Depend on other signals and react to their changes
/// - Have other signals depend on it

#[macro_export]
macro_rules! __signal_aux {
    ([self] $var:ident, $_self:ident) => {
        let $var = $_self.get();
    };
    ($var:ident, $_self:ident) => {
        let $var = $var.get();
    };
}

/// A macro to create reactive signals.
///
/// This macro supports creating both single-threaded signals (`reactivity::Signal`)
/// and thread-safe signals (`reactivity::sync::Signal`) depending on the context.
///
/// # Syntax
///
/// ```
/// // Create a basic signal with a value
/// signal!(value)
///
/// // Create a signal that reacts to other signals
/// signal!([dep1, dep2, ...] expression)
///
/// // Create a signal with custom effect function
/// signal!(<before, after> [dep1, dep2, ...] expression; effect_code)
/// ```
///
/// # Examples
///
/// ## Single-threaded usage
///
/// ```rust
/// use reactivity::Signal;
/// let x = signal!(1);
/// let y = signal!([x] x * 2);
///
/// x.send(5);
/// assert_eq!(y.get(), 10);
/// ```
///
/// ## Thread-safe usage
///
/// ```rust
/// use std::thread;
/// use reactivity::sync::Signal;
///
/// let x = signal!(1);
/// let y = signal!([x] x * 2);
/// let x_clone = x.clone();
///
/// thread::spawn(move || {
///     x_clone.send(5);
/// }).join().unwrap();
///
/// assert_eq!(y.get(), 10);
/// ```
///
/// # Choosing Between Signal Types
///
/// - Use `reactivity::Signal` (imported with `use reactivity::Signal`) for single-threaded contexts
/// - Use `reactivity::sync::Signal` (imported with `use reactivity::sync::Signal`) for multi-threaded contexts
///
/// The `signal!` macro will use the correct Signal implementation based on your imports.
#[macro_export]
macro_rules! signal {
    ($(< $_before:ident $(, $_after:ident)? >)? [$($params:ident),*] $proc:expr) => {
        signal!($(<$_before:ident $(, $_after:ident)?>)? [$($params),*] $proc; ())
    };
    ($(< $_before:ident $(, $_after:ident)? >)? [$($params:ident),*] $proc:expr; $eff:expr) => {
        {
            use $crate::api::SignalTrait;
            $(
                let $params = $params.clone();
                paste::paste!{ let [<$params _>] = $params.clone(); }
                paste::paste!{ let [<$params __>] = $params.clone(); }
            )*
            let processor = move || {
                $(
                    let $params = $params.get();
                )*
                $proc
            };
            let signal = Signal::driven(processor, move |_self, _after| {
                $(
                    let $_before = _self.get();
                    $(
                        let $_after = _after.clone();
                    )?
                )?
                $(
                    paste::paste!{
                        #[allow(unused_variables)]
                        let $params = [<$params _>].clone();
                    }
                )*
                $eff
            });

            $(
                paste::paste!{
                    let signal_ = signal.clone();
                    [<$params __>].add_receiver(signal_);
                }
            )*

            signal
        }
    };

    ($value:expr) => {
        {
            use $crate::api::SignalTrait;
            Signal::new($value)
        }
    };
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{api::SignalTrait, sync::Signal};

    #[test]
    fn test() {
        // Diamond dependency
        let x = signal!(1);
        let doubled_x = signal!([x] x * 2);
        let tripled_x = signal!([x] x * 3);
        let _ = signal!(
            <before, now> 
            [doubled_x, tripled_x] 
            doubled_x + tripled_x; 
            println!("output {before} -> {now}"));
        thread::spawn(move || loop {
            x.send(x.get() + 1);
            thread::sleep(std::time::Duration::from_millis(100));
        })
        .join()
        .unwrap();
    }
}
