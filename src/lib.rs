pub mod api;
pub mod sync;

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
#[macro_export]
macro_rules! sync_signal {
    ($(< $_before:ident $(, $_after:ident)? >)? [$($params:ident),*] $proc:expr) => {
        sync_signal!($(<$_before:ident $(, $_after:ident)?>)? [$($params),*] $proc; ())
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
            let signal = SyncSignal::driven(processor, move |_self, _after| {
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
        SyncSignal::new($value)
    };
}

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
    use std::thread::{self, sleep};

    use crate::sync::SyncSignal;

    #[test]
    fn test() {
        // Diamond dependency
        let x = sync_signal!(1);
        let doubled_x = sync_signal!([x] x * 2);
        let tripled_x = sync_signal!([x] x * 3);
        let _ = sync_signal!(
            <before, now> 
            [doubled_x, tripled_x] 
            doubled_x + tripled_x; 
            println!("output {before} -> {now}"));

        thread::spawn(move || loop {
            sleep(std::time::Duration::from_secs(1));
            x.send(x.get() + 1);
        })
        .join()
        .unwrap();
    }
}
