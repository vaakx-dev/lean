#[cfg(not(feature = "terminal"))]
mod disabled;
#[cfg(feature = "terminal")]
mod dock;
#[cfg(feature = "terminal")]
mod resize;

#[cfg(not(feature = "terminal"))]
pub(super) use disabled::TerminalDock;
#[cfg(feature = "terminal")]
pub(super) use dock::TerminalDock;
