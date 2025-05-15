use std::sync::Arc;

use api::SignalBase;

pub mod api;

/// A reactive signal that can be observed and updated.
/// It is thread-safe and can be used in concurrent environments.
///
/// `Signal<T>` is a type alias for `SignalBase<T, Arc<parking_lot::RwLock<T>>>`,
/// which provides a thread-safe implementation of the reactive signal.
///
/// It can:
/// - Hold a value that can be read with `get()` or `borrow()`
/// - Be updated with new values via `send()`
/// - Depend on other signals and react to their changes
/// - Have other signals depend on it
///
/// # Examples
///
/// ```rust
/// use reactivity::Signal;
///
/// // Create a new signal
/// let count = Signal::new(0);
///
/// // Read the value
/// assert_eq!(count.get(), 0);
///
/// // Update the value
/// count.send(42);
/// assert_eq!(count.get(), 42);
/// ```
pub type Signal<T> = SignalBase<T, Arc<parking_lot::RwLock<T>>>;

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
/// ```rust
/// # use reactivity::{signal, Signal};
/// let x = signal!(1);
/// let y = signal!(<_y, y> [x] x + 2; println!("y {_y} -> {y}"));
/// let z = signal!(<_z, z> [y] y * y; println!("z {_z} -> {z}"));
///
/// x.send(2);
///
/// assert_eq!(x.get(), 2);
/// assert_eq!(y.get(), 4);
/// assert_eq!(z.get(), 16);
///
/// // Output:
/// // y 3 -> 4
/// // z 9 -> 16
/// ```
///
/// ## Parameters
///
/// - `<before, after>`: Optional identifiers to capture the previous (`before`) and
///   new (`after`) values when the signal changes
/// - `[dep1, dep2, ...]`: Dependencies - signals this signal reacts to
/// - `expression`: Expression that computes the new value
/// - `effect_code`: Optional side effect code executed when the signal changes
///
/// # Best Practices
///
/// Always use the `signal!` macro to create signals instead of using `Signal::new`
/// or `Signal::driven` directly. The macro automatically sets up the dependency chain
/// by calling `add_receiver` for each dependency.
///
/// # Dependency Tracking
///
/// The macro automatically tracks dependencies between signals:
///
/// ```rust
/// # use reactivity::{signal, Signal};
/// let a = signal!(1);
/// let b = signal!(2);
/// let sum = signal!([a, b] a + b);
///
/// // When a or b changes, sum will automatically update
/// a.send(10);
/// assert_eq!(sum.get(), 12);
///
/// b.send(20);
/// assert_eq!(sum.get(), 30);
/// ```

#[macro_export]
macro_rules! signal {
    ($(< $_before:ident $(, $_after:ident)? >)? [$($params:ident),*] $proc:expr) => {
        signal!($(<$_before:ident $(, $_after:ident)?>)? [$($params),*] $proc; ())
    };
    ($(< $_before:ident $(, $_after:ident)? >)? [$($params:ident),*] $proc:expr; $eff:expr) => {
        {
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
                    [<$params __>].add_receiver(&signal);
                }
            )*

            signal
        }
    };

    ($value:expr) => {
        Signal::new($value)
    };
}

#[cfg(test)]
mod tests {
    use crate::Signal;

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
        x.send(2);
    }
}
