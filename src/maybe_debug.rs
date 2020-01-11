/// Trait to print values using `Debug` if it is implemented.
pub trait MaybeDebug {
    /// Test if debug is implemented for the type.
    fn is_debug() -> bool;

    /// Format self, or print a fallback if the type does not implement Debug.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

/// Default implementation of MaybeDebug that prints a fallback value.
impl<T> MaybeDebug for T {
    default fn is_debug() -> bool {
        false
    }

    default fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<object of type {}>", std::any::type_name::<Self>())
    }
}

/// Specilization of MaybeDebug for types that implement Debug.
impl<T: std::fmt::Debug> MaybeDebug for T {
    fn is_debug() -> bool {
        true
    }

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// Wrapper that always implements Debug using the MaybeDebug trait.
pub struct DebugWrapper<'a, T>(&'a T);

/// Print the wrapped value if it implements Debug, or a fallback.
impl<T: MaybeDebug> std::fmt::Debug for DebugWrapper<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        MaybeDebug::fmt(self.0, f)
    }
}

/// Wrap a value so that it can always be printed with Debug.
pub fn wrap<T: MaybeDebug>(value: &T) -> DebugWrapper<T> {
    DebugWrapper(value)
}
