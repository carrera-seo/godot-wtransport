# Architecture

The networking core owns a Tokio runtime and exposes nonblocking commands to
the host. Sessions and streams use monotonically increasing 64-bit handles.
Handles are never reused during a client instance lifetime, preventing stale
Godot objects from addressing a newly allocated transport resource.

Events cross the runtime boundary through a bounded channel. Reliable events
wait for queue capacity and therefore apply backpressure to stream reads.
Unreliable datagrams use a nonblocking enqueue operation and may be discarded
when the host does not poll quickly enough. The dropped count is observable in
client statistics.

The C ABI is the only boundary used by the future C++ adapter. Every exported
entry point catches Rust panics. Variable-sized event data is transferred in a
single allocation and must be released with `gwt_event_free`.
