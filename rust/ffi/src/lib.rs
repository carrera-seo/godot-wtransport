use std::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::slice;

use godot_wtransport_core::{CertificateMode, Client, ClientError, ConnectOptions, Event};

pub const ABI_VERSION: u32 = 1;

#[repr(C)]
pub struct GwtClient {
    client: Client,
}

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum GwtStatus {
    Ok = 0,
    NoEvent = 1,
    InvalidArgument = -1,
    InvalidHandle = -2,
    InternalError = -3,
    Panic = -4,
}

#[repr(C)]
pub struct GwtEvent {
    pub kind: u32,
    pub error_category: u32,
    pub session: u64,
    pub stream: u64,
    pub code: i64,
    pub retryable: bool,
    pub transport_error_code: u64,
    pub http3_error_code: u64,
    pub stream_error_code: u64,
    pub tls_alert: u8,
    pub os_error: i32,
    pub data: *mut u8,
    pub data_len: usize,
}

impl Default for GwtEvent {
    fn default() -> Self {
        Self {
            kind: 0,
            error_category: 0,
            session: 0,
            stream: 0,
            code: 0,
            retryable: false,
            transport_error_code: 0,
            http3_error_code: 0,
            stream_error_code: 0,
            tls_alert: 0,
            os_error: 0,
            data: ptr::null_mut(),
            data_len: 0,
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct GwtClientStats {
    pub dropped_datagrams: u64,
    pub queued_events: u64,
    pub active_sessions: u64,
    pub active_streams: u64,
}

fn status(error: ClientError) -> GwtStatus {
    match error {
        ClientError::InvalidArgument(_) => GwtStatus::InvalidArgument,
        ClientError::InvalidSession | ClientError::InvalidStream => GwtStatus::InvalidHandle,
        ClientError::ShuttingDown => GwtStatus::InternalError,
    }
}

fn guarded(function: impl FnOnce() -> GwtStatus) -> GwtStatus {
    catch_unwind(AssertUnwindSafe(function)).unwrap_or(GwtStatus::Panic)
}

unsafe fn client_ref<'a>(client: *mut GwtClient) -> Option<&'a GwtClient> {
    unsafe { client.as_ref() }
}

unsafe fn bytes<'a>(data: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        Some(&[])
    } else if data.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(data, len) })
    }
}

unsafe fn string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

#[unsafe(no_mangle)]
pub extern "C" fn gwt_abi_version() -> u32 {
    ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn gwt_client_create(event_capacity: usize) -> *mut GwtClient {
    catch_unwind(AssertUnwindSafe(|| {
        Client::new(event_capacity)
            .ok()
            .map(|client| Box::into_raw(Box::new(GwtClient { client })))
            .unwrap_or(ptr::null_mut())
    }))
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
/// Destroys a client allocated by [`gwt_client_create`].
///
/// # Safety
///
/// `client` must be null or a live pointer returned by `gwt_client_create`, and
/// it must not be used after this call.
pub unsafe extern "C" fn gwt_client_destroy(client: *mut GwtClient) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !client.is_null() {
            drop(unsafe { Box::from_raw(client) });
        }
    }));
}

#[unsafe(no_mangle)]
/// Starts a connection using native certificate roots.
///
/// # Safety
///
/// All pointers must be valid for their documented types. `url` must be a
/// NUL-terminated UTF-8 string and `out_session` must be writable.
pub unsafe extern "C" fn gwt_client_connect(
    client: *mut GwtClient,
    url: *const c_char,
    out_session: *mut u64,
) -> GwtStatus {
    guarded(|| {
        let Some(client) = (unsafe { client_ref(client) }) else {
            return GwtStatus::InvalidArgument;
        };
        let Some(url) = (unsafe { string(url) }) else {
            return GwtStatus::InvalidArgument;
        };
        let Some(out_session) = (unsafe { out_session.as_mut() }) else {
            return GwtStatus::InvalidArgument;
        };
        match client.client.connect(url, ConnectOptions::default()) {
            Ok(session) => {
                *out_session = session;
                GwtStatus::Ok
            }
            Err(error) => status(error),
        }
    })
}

#[unsafe(no_mangle)]
/// Starts a connection using one or more SHA-256 certificate hashes.
///
/// # Safety
///
/// `hashes` must reference `hash_count * 32` readable bytes. Other pointers
/// follow the requirements of [`gwt_client_connect`].
pub unsafe extern "C" fn gwt_client_connect_hashes(
    client: *mut GwtClient,
    url: *const c_char,
    hashes: *const u8,
    hash_count: usize,
    out_session: *mut u64,
) -> GwtStatus {
    guarded(|| {
        let Some(client) = (unsafe { client_ref(client) }) else {
            return GwtStatus::InvalidArgument;
        };
        let Some(url) = (unsafe { string(url) }) else {
            return GwtStatus::InvalidArgument;
        };
        let Some(out_session) = (unsafe { out_session.as_mut() }) else {
            return GwtStatus::InvalidArgument;
        };
        let Some(raw_hashes) = (unsafe { bytes(hashes, hash_count.saturating_mul(32)) }) else {
            return GwtStatus::InvalidArgument;
        };
        let parsed = raw_hashes
            .chunks_exact(32)
            .map(|hash| <[u8; 32]>::try_from(hash).expect("chunk size is fixed"))
            .collect();
        let options = ConnectOptions {
            certificate_mode: CertificateMode::ServerCertificateHashes(parsed),
            ..ConnectOptions::default()
        };
        match client.client.connect(url, options) {
            Ok(session) => {
                *out_session = session;
                GwtStatus::Ok
            }
            Err(error) => status(error),
        }
    })
}

#[unsafe(no_mangle)]
/// Sends an unreliable datagram.
///
/// # Safety
///
/// `client` must be live and `data` must reference `data_len` readable bytes,
/// unless `data_len` is zero.
pub unsafe extern "C" fn gwt_client_send_datagram(
    client: *mut GwtClient,
    session: u64,
    data: *const u8,
    data_len: usize,
) -> GwtStatus {
    guarded(|| {
        let (Some(client), Some(data)) = (unsafe { client_ref(client) }, unsafe {
            bytes(data, data_len)
        }) else {
            return GwtStatus::InvalidArgument;
        };
        client
            .client
            .send_datagram(session, data)
            .map_or_else(status, |_| GwtStatus::Ok)
    })
}

#[unsafe(no_mangle)]
/// Opens a bidirectional stream.
///
/// # Safety
///
/// `client` must be live and `out_stream` must be writable.
pub unsafe extern "C" fn gwt_client_open_bidirectional_stream(
    client: *mut GwtClient,
    session: u64,
    out_stream: *mut u64,
) -> GwtStatus {
    unsafe { open_stream(client, session, out_stream, true) }
}

#[unsafe(no_mangle)]
/// Opens a unidirectional stream.
///
/// # Safety
///
/// `client` must be live and `out_stream` must be writable.
pub unsafe extern "C" fn gwt_client_open_unidirectional_stream(
    client: *mut GwtClient,
    session: u64,
    out_stream: *mut u64,
) -> GwtStatus {
    unsafe { open_stream(client, session, out_stream, false) }
}

unsafe fn open_stream(
    client: *mut GwtClient,
    session: u64,
    out_stream: *mut u64,
    bidirectional: bool,
) -> GwtStatus {
    guarded(|| {
        let (Some(client), Some(out_stream)) = (unsafe { client_ref(client) }, unsafe {
            out_stream.as_mut()
        }) else {
            return GwtStatus::InvalidArgument;
        };
        let result = if bidirectional {
            client.client.open_bidirectional_stream(session)
        } else {
            client.client.open_unidirectional_stream(session)
        };
        match result {
            Ok(stream) => {
                *out_stream = stream;
                GwtStatus::Ok
            }
            Err(error) => status(error),
        }
    })
}

#[unsafe(no_mangle)]
/// Enqueues bytes for a writable stream.
///
/// # Safety
///
/// `client` must be live and `data` must reference `data_len` readable bytes,
/// unless `data_len` is zero.
pub unsafe extern "C" fn gwt_client_write_stream(
    client: *mut GwtClient,
    stream: u64,
    data: *const u8,
    data_len: usize,
) -> GwtStatus {
    guarded(|| {
        let (Some(client), Some(data)) = (unsafe { client_ref(client) }, unsafe {
            bytes(data, data_len)
        }) else {
            return GwtStatus::InvalidArgument;
        };
        client
            .client
            .write_stream(stream, data.to_vec())
            .map_or_else(status, |_| GwtStatus::Ok)
    })
}

#[unsafe(no_mangle)]
/// Finishes the sending side of a stream.
///
/// # Safety
///
/// `client` must be a live pointer returned by `gwt_client_create`.
pub unsafe extern "C" fn gwt_client_finish_stream(
    client: *mut GwtClient,
    stream: u64,
) -> GwtStatus {
    guarded(|| {
        let Some(client) = (unsafe { client_ref(client) }) else {
            return GwtStatus::InvalidArgument;
        };
        client
            .client
            .finish_stream(stream)
            .map_or_else(status, |_| GwtStatus::Ok)
    })
}

#[unsafe(no_mangle)]
/// Closes a WebTransport session.
///
/// # Safety
///
/// `client` must be live and `reason` must reference `reason_len` readable
/// bytes, unless `reason_len` is zero.
pub unsafe extern "C" fn gwt_client_close(
    client: *mut GwtClient,
    session: u64,
    code: u32,
    reason: *const u8,
    reason_len: usize,
) -> GwtStatus {
    guarded(|| {
        let (Some(client), Some(reason)) = (unsafe { client_ref(client) }, unsafe {
            bytes(reason, reason_len)
        }) else {
            return GwtStatus::InvalidArgument;
        };
        client
            .client
            .close(session, code, reason)
            .map_or_else(status, |_| GwtStatus::Ok)
    })
}

#[unsafe(no_mangle)]
/// Polls one event without blocking.
///
/// # Safety
///
/// `client` must be live and `out_event` must be writable. A returned event
/// must eventually be released with [`gwt_event_free`].
pub unsafe extern "C" fn gwt_client_poll(
    client: *mut GwtClient,
    out_event: *mut GwtEvent,
) -> GwtStatus {
    guarded(|| {
        let (Some(client), Some(out_event)) =
            (unsafe { client_ref(client) }, unsafe { out_event.as_mut() })
        else {
            return GwtStatus::InvalidArgument;
        };
        let Some(event) = client.client.poll() else {
            return GwtStatus::NoEvent;
        };
        *out_event = into_ffi_event(event);
        GwtStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Copies the current client statistics.
///
/// # Safety
///
/// `client` must be live and `out_stats` must be writable.
pub unsafe extern "C" fn gwt_client_stats(
    client: *mut GwtClient,
    out_stats: *mut GwtClientStats,
) -> GwtStatus {
    guarded(|| {
        let (Some(client), Some(out_stats)) =
            (unsafe { client_ref(client) }, unsafe { out_stats.as_mut() })
        else {
            return GwtStatus::InvalidArgument;
        };
        let stats = client.client.stats();
        *out_stats = GwtClientStats {
            dropped_datagrams: stats.dropped_datagrams,
            queued_events: stats.queued_events,
            active_sessions: stats.active_sessions,
            active_streams: stats.active_streams,
        };
        GwtStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Releases data owned by a polled event.
///
/// # Safety
///
/// `event` must be null or point to a value initialized by
/// [`gwt_client_poll`]. Each event may be freed only once.
pub unsafe extern "C" fn gwt_event_free(event: *mut GwtEvent) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let Some(event) = (unsafe { event.as_mut() }) else {
            return;
        };
        if !event.data.is_null() {
            drop(unsafe { Vec::from_raw_parts(event.data, event.data_len, event.data_len) });
        }
        *event = GwtEvent::default();
    }));
}

fn into_ffi_event(event: Event) -> GwtEvent {
    let error = event.error;
    let mut data = event.data;
    data.shrink_to_fit();
    let ffi = GwtEvent {
        kind: event.kind as u32,
        error_category: error.as_ref().map_or(0, |value| value.category as u32),
        session: event.session,
        stream: event.stream,
        code: event.code,
        retryable: error.as_ref().is_some_and(|value| value.retryable),
        transport_error_code: error
            .as_ref()
            .and_then(|value| value.transport_error_code)
            .unwrap_or(0),
        http3_error_code: error
            .as_ref()
            .and_then(|value| value.http3_error_code)
            .unwrap_or(0),
        stream_error_code: error
            .as_ref()
            .and_then(|value| value.stream_error_code)
            .unwrap_or(0),
        tls_alert: error
            .as_ref()
            .and_then(|value| value.tls_alert)
            .unwrap_or(0),
        os_error: error.as_ref().and_then(|value| value.os_error).unwrap_or(0),
        data: data.as_mut_ptr(),
        data_len: data.len(),
    };
    std::mem::forget(data);
    ffi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_abi_version_matches_rust() {
        let header = include_str!("../include/godot_wtransport.h");
        assert!(header.contains(&format!("#define GWT_ABI_VERSION {ABI_VERSION}u")));
    }

    #[test]
    fn null_arguments_do_not_panic() {
        assert_eq!(
            unsafe { gwt_client_connect(ptr::null_mut(), ptr::null(), ptr::null_mut()) } as i32,
            GwtStatus::InvalidArgument as i32
        );
        unsafe { gwt_client_destroy(ptr::null_mut()) };
        unsafe { gwt_event_free(ptr::null_mut()) };
    }

    #[test]
    fn repeated_ffi_lifecycle_is_safe() {
        for _ in 0..100 {
            let client = gwt_client_create(8);
            assert!(!client.is_null());
            unsafe { gwt_client_destroy(client) };
        }
    }
}
