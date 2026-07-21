//! First-class owned host resources.
//!
//! [`OwnedResource`] carries one file, process, socket, HTTP body/writer, task,
//! timer, or bounded channel behind [`Value::Resource`](crate::value::Value).
//! Resources are move-only language values with explicit close, cancellation,
//! deadlines, and pressure-aware nonblocking operations.
//!
//! # Usage
//!
//! ```
//! use tetherscript::value::resource::{OwnedResource, ResourceKind};
//!
//! let resource = OwnedResource::channel(2)?;
//! assert_eq!(resource.kind(), ResourceKind::Channel);
//! # Ok::<(), String>(())
//! ```

mod args;
mod availability;
mod channel_queue;
mod construct;
mod control;
mod deadline;
mod dispatch;
mod display;
mod factory;
mod factory_memory;
mod factory_os;
mod factory_render;
mod install;
mod kind;
mod lifecycle;
mod method_control;
mod native;
mod owned;
mod payload;
mod payload_call;
mod payload_cancel;
mod payload_kind;
mod result;

mod channel;
mod child_process;
mod file;
mod render_surface;
mod render_surface_construct;
mod render_surface_output;
mod render_surface_render;
mod request_body;
mod response_writer;
mod response_writer_buffer;
mod task;
mod task_result;
mod tcp_listener;
mod tcp_stream;
mod timer;
pub(crate) mod transfer;

#[cfg(test)]
mod tests;

pub use kind::ResourceKind;
pub use owned::OwnedResource;

pub(crate) use install::install;
