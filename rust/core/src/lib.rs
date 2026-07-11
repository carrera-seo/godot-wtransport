use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use thiserror::Error;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Mutex as AsyncMutex, mpsc};
use wtransport::tls::Sha256Digest;
use wtransport::{ClientConfig, Connection, Endpoint, VarInt};

pub type SessionHandle = u64;
pub type StreamHandle = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EventKind {
    Connected = 1,
    ConnectionFailed = 2,
    Closed = 3,
    Datagram = 4,
    StreamOpened = 5,
    IncomingBidirectionalStream = 6,
    IncomingUnidirectionalStream = 7,
    StreamData = 8,
    StreamFinished = 9,
    StreamReset = 10,
    Error = 11,
    Draining = 12,
    Trace = 13,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ErrorCategory {
    None = 0,
    InvalidArgument = 1,
    Dns = 2,
    Tls = 3,
    Transport = 4,
    Http3 = 5,
    Session = 6,
    Stream = 7,
    Queue = 8,
    Internal = 9,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportError {
    pub category: ErrorCategory,
    pub code: i64,
    pub message: String,
    pub retryable: bool,
    pub transport_error_code: Option<u64>,
    pub http3_error_code: Option<u64>,
    pub stream_error_code: Option<u64>,
    pub tls_alert: Option<u8>,
    pub os_error: Option<i32>,
}

impl TransportError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::InvalidArgument,
            code: -1,
            message: message.into(),
            retryable: false,
            transport_error_code: None,
            http3_error_code: None,
            stream_error_code: None,
            tls_alert: None,
            os_error: None,
        }
    }

    fn from_network(error: impl std::fmt::Display) -> Self {
        let message = error.to_string();
        let lower = message.to_ascii_lowercase();
        let category = if lower.contains("certificate") || lower.contains("tls") {
            ErrorCategory::Tls
        } else if lower.contains("dns") || lower.contains("name") {
            ErrorCategory::Dns
        } else {
            ErrorCategory::Transport
        };
        Self {
            category,
            code: -2,
            message,
            retryable: category != ErrorCategory::Tls,
            transport_error_code: None,
            http3_error_code: None,
            stream_error_code: None,
            tls_alert: None,
            os_error: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub kind: EventKind,
    pub session: SessionHandle,
    pub stream: StreamHandle,
    pub code: i64,
    pub data: Vec<u8>,
    pub error: Option<TransportError>,
}

impl Event {
    fn simple(kind: EventKind, session: SessionHandle) -> Self {
        Self {
            kind,
            session,
            stream: 0,
            code: 0,
            data: Vec::new(),
            error: None,
        }
    }

    fn failure(session: SessionHandle, error: TransportError) -> Self {
        Self {
            kind: EventKind::ConnectionFailed,
            session,
            stream: 0,
            code: error.code,
            data: error.message.as_bytes().to_vec(),
            error: Some(error),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CertificateMode {
    NativeRoots,
    ServerCertificateHashes(Vec<[u8; 32]>),
    CustomCaCertificates(Vec<Vec<u8>>),
    #[cfg(feature = "dangerous-insecure")]
    Insecure,
}

#[derive(Clone, Debug)]
pub struct ConnectOptions {
    pub certificate_mode: CertificateMode,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            certificate_mode: CertificateMode::NativeRoots,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ClientStats {
    pub dropped_datagrams: u64,
    pub queued_events: u64,
    pub active_sessions: u64,
    pub active_streams: u64,
    pub active_draining_sessions: u64,
    pub datagrams_sent: u64,
    pub datagrams_received: u64,
    pub stream_bytes_sent: u64,
    pub stream_bytes_received: u64,
    pub connection_failures: u64,
    pub dropped_trace_events: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SessionState {
    Connecting = 1,
    Connected = 2,
    Draining = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionDiagnostics {
    pub state: SessionState,
    pub stable_id: u64,
    pub rtt_micros: u64,
    pub max_datagram_size: u64,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("session handle is not active")]
    InvalidSession,
    #[error("stream handle is not active")]
    InvalidStream,
    #[error("runtime is shutting down")]
    ShuttingDown,
}

struct EventQueue {
    sender: mpsc::Sender<Event>,
    receiver: Mutex<mpsc::Receiver<Event>>,
    queued: AtomicU64,
    dropped_datagrams: AtomicU64,
    dropped_trace_events: AtomicU64,
}

impl EventQueue {
    fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity.max(1));
        Self {
            sender,
            receiver: Mutex::new(receiver),
            queued: AtomicU64::new(0),
            dropped_datagrams: AtomicU64::new(0),
            dropped_trace_events: AtomicU64::new(0),
        }
    }

    fn send_trace(
        &self,
        enabled: bool,
        name: &'static str,
        session: SessionHandle,
        stream: StreamHandle,
        value: u64,
    ) {
        if !enabled {
            return;
        }
        let event = Event {
            kind: EventKind::Trace,
            session,
            stream,
            code: i64::try_from(value).unwrap_or(i64::MAX),
            data: name.as_bytes().to_vec(),
            error: None,
        };
        match self.sender.try_send(event) {
            Ok(()) => {
                self.queued.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                self.dropped_trace_events.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    async fn send_reliable(&self, event: Event) {
        if self.sender.send(event).await.is_ok() {
            self.queued.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn send_datagram(&self, event: Event) {
        match self.sender.try_send(event) {
            Ok(()) => {
                self.queued.fetch_add(1, Ordering::Relaxed);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped_datagrams.fetch_add(1, Ordering::Relaxed);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    }

    fn poll(&self) -> Option<Event> {
        let event = self.receiver.lock().ok()?.try_recv().ok()?;
        self.queued.fetch_sub(1, Ordering::Relaxed);
        Some(event)
    }
}

struct Shared {
    events: EventQueue,
    sessions: Mutex<HashMap<SessionHandle, Arc<Connection>>>,
    session_states: Mutex<HashMap<SessionHandle, SessionState>>,
    send_streams: Mutex<HashMap<StreamHandle, Arc<AsyncMutex<wtransport::SendStream>>>>,
    next_session: AtomicU64,
    next_stream: AtomicU64,
    trace_enabled: AtomicBool,
    datagrams_sent: AtomicU64,
    datagrams_received: AtomicU64,
    stream_bytes_sent: AtomicU64,
    stream_bytes_received: AtomicU64,
    connection_failures: AtomicU64,
}

impl Shared {
    fn next_session(&self) -> SessionHandle {
        self.next_session.fetch_add(1, Ordering::Relaxed)
    }

    fn next_stream(&self) -> StreamHandle {
        self.next_stream.fetch_add(1, Ordering::Relaxed)
    }

    fn session(&self, handle: SessionHandle) -> Option<Arc<Connection>> {
        self.sessions.lock().ok()?.get(&handle).cloned()
    }

    fn state(&self, handle: SessionHandle) -> Option<SessionState> {
        self.session_states.lock().ok()?.get(&handle).copied()
    }

    fn is_draining(&self, handle: SessionHandle) -> bool {
        self.state(handle) == Some(SessionState::Draining)
    }

    fn trace(&self, name: &'static str, session: SessionHandle, stream: StreamHandle, value: u64) {
        self.events.send_trace(
            self.trace_enabled.load(Ordering::Relaxed),
            name,
            session,
            stream,
            value,
        );
    }
}

pub struct Client {
    runtime: Option<Runtime>,
    shared: Arc<Shared>,
}

impl Client {
    pub fn new(event_capacity: usize) -> Result<Self, ClientError> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("godot-wtransport")
            .build()
            .map_err(|error| ClientError::InvalidArgument(error.to_string()))?;
        Ok(Self {
            runtime: Some(runtime),
            shared: Arc::new(Shared {
                events: EventQueue::new(event_capacity),
                sessions: Mutex::new(HashMap::new()),
                session_states: Mutex::new(HashMap::new()),
                send_streams: Mutex::new(HashMap::new()),
                next_session: AtomicU64::new(1),
                next_stream: AtomicU64::new(1),
                trace_enabled: AtomicBool::new(false),
                datagrams_sent: AtomicU64::new(0),
                datagrams_received: AtomicU64::new(0),
                stream_bytes_sent: AtomicU64::new(0),
                stream_bytes_received: AtomicU64::new(0),
                connection_failures: AtomicU64::new(0),
            }),
        })
    }

    pub fn connect(
        &self,
        url: impl Into<String>,
        options: ConnectOptions,
    ) -> Result<SessionHandle, ClientError> {
        let url = url.into();
        if !url.starts_with("https://") {
            return Err(ClientError::InvalidArgument(
                "WebTransport URLs must use https://".into(),
            ));
        }
        let runtime = self.runtime.as_ref().ok_or(ClientError::ShuttingDown)?;
        let session = self.shared.next_session();
        if let Ok(mut states) = self.shared.session_states.lock() {
            states.insert(session, SessionState::Connecting);
        }
        self.shared.trace("connect_started", session, 0, 0);
        let shared = Arc::clone(&self.shared);
        runtime.spawn(async move {
            if let Err(error) = connect_task(Arc::clone(&shared), session, url, options).await {
                shared.connection_failures.fetch_add(1, Ordering::Relaxed);
                shared.trace("connection_failed", session, 0, 0);
                shared
                    .events
                    .send_reliable(Event::failure(session, error))
                    .await;
            }
            if let Ok(mut sessions) = shared.sessions.lock() {
                sessions.remove(&session);
            }
            if let Ok(mut states) = shared.session_states.lock() {
                states.remove(&session);
            }
        });
        Ok(session)
    }

    pub fn send_datagram(&self, session: SessionHandle, data: &[u8]) -> Result<(), ClientError> {
        if self.shared.is_draining(session) {
            return Err(ClientError::InvalidSession);
        }
        let connection = self
            .shared
            .session(session)
            .ok_or(ClientError::InvalidSession)?;
        connection
            .send_datagram(data)
            .map_err(|error| ClientError::InvalidArgument(error.to_string()))?;
        self.shared.datagrams_sent.fetch_add(1, Ordering::Relaxed);
        self.shared
            .trace("datagram_sent", session, 0, data.len() as u64);
        Ok(())
    }

    pub fn open_bidirectional_stream(
        &self,
        session: SessionHandle,
    ) -> Result<StreamHandle, ClientError> {
        if self.shared.is_draining(session) {
            return Err(ClientError::InvalidSession);
        }
        let connection = self
            .shared
            .session(session)
            .ok_or(ClientError::InvalidSession)?;
        let runtime = self.runtime.as_ref().ok_or(ClientError::ShuttingDown)?;
        let stream = self.shared.next_stream();
        let shared = Arc::clone(&self.shared);
        runtime.spawn(async move {
            match connection.open_bi().await {
                Ok(opening) => match opening.await {
                    Ok((send, recv)) => {
                        if let Ok(mut streams) = shared.send_streams.lock() {
                            streams.insert(stream, Arc::new(AsyncMutex::new(send)));
                        }
                        shared
                            .events
                            .send_reliable(Event {
                                kind: EventKind::StreamOpened,
                                session,
                                stream,
                                code: 0,
                                data: Vec::new(),
                                error: None,
                            })
                            .await;
                        spawn_receive_stream(Arc::clone(&shared), session, stream, recv);
                    }
                    Err(error) => send_stream_error(&shared, session, stream, error).await,
                },
                Err(error) => send_stream_error(&shared, session, stream, error).await,
            }
        });
        Ok(stream)
    }

    pub fn open_unidirectional_stream(
        &self,
        session: SessionHandle,
    ) -> Result<StreamHandle, ClientError> {
        if self.shared.is_draining(session) {
            return Err(ClientError::InvalidSession);
        }
        let connection = self
            .shared
            .session(session)
            .ok_or(ClientError::InvalidSession)?;
        let runtime = self.runtime.as_ref().ok_or(ClientError::ShuttingDown)?;
        let stream = self.shared.next_stream();
        let shared = Arc::clone(&self.shared);
        runtime.spawn(async move {
            match connection.open_uni().await {
                Ok(opening) => match opening.await {
                    Ok(send) => {
                        if let Ok(mut streams) = shared.send_streams.lock() {
                            streams.insert(stream, Arc::new(AsyncMutex::new(send)));
                        }
                        shared
                            .events
                            .send_reliable(Event {
                                kind: EventKind::StreamOpened,
                                session,
                                stream,
                                code: 0,
                                data: Vec::new(),
                                error: None,
                            })
                            .await;
                    }
                    Err(error) => send_stream_error(&shared, session, stream, error).await,
                },
                Err(error) => send_stream_error(&shared, session, stream, error).await,
            }
        });
        Ok(stream)
    }

    pub fn write_stream(&self, stream: StreamHandle, data: Vec<u8>) -> Result<(), ClientError> {
        let runtime = self.runtime.as_ref().ok_or(ClientError::ShuttingDown)?;
        let shared = Arc::clone(&self.shared);
        let send = self
            .shared
            .send_streams
            .lock()
            .ok()
            .and_then(|streams| streams.get(&stream).cloned())
            .ok_or(ClientError::InvalidStream)?;
        let size = data.len() as u64;
        runtime.spawn(async move {
            if let Err(error) = send.lock().await.write_all(&data).await {
                send_stream_error(&shared, 0, stream, error).await;
            } else {
                shared.stream_bytes_sent.fetch_add(size, Ordering::Relaxed);
                shared.trace("stream_write_completed", 0, stream, size);
            }
        });
        Ok(())
    }

    pub fn finish_stream(&self, stream: StreamHandle) -> Result<(), ClientError> {
        let runtime = self.runtime.as_ref().ok_or(ClientError::ShuttingDown)?;
        let send = self
            .shared
            .send_streams
            .lock()
            .ok()
            .and_then(|mut streams| streams.remove(&stream))
            .ok_or(ClientError::InvalidStream)?;
        runtime.spawn(async move {
            let _ = send.lock().await.finish().await;
        });
        Ok(())
    }

    pub fn close(
        &self,
        session: SessionHandle,
        code: u32,
        reason: &[u8],
    ) -> Result<(), ClientError> {
        let connection = self
            .shared
            .session(session)
            .ok_or(ClientError::InvalidSession)?;
        connection.close(VarInt::from_u32(code), reason);
        self.shared
            .trace("session_close_requested", session, 0, code as u64);
        Ok(())
    }

    pub fn drain(
        &self,
        session: SessionHandle,
        timeout: Duration,
        code: u32,
        reason: Vec<u8>,
    ) -> Result<(), ClientError> {
        if timeout.is_zero() {
            return Err(ClientError::InvalidArgument(
                "drain timeout must be positive".into(),
            ));
        }
        let connection = self
            .shared
            .session(session)
            .ok_or(ClientError::InvalidSession)?;
        let runtime = self.runtime.as_ref().ok_or(ClientError::ShuttingDown)?;
        let changed = self
            .shared
            .session_states
            .lock()
            .map(|mut states| {
                if states.get(&session) == Some(&SessionState::Connected) {
                    states.insert(session, SessionState::Draining);
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);
        if !changed {
            return Err(ClientError::InvalidSession);
        }
        self.shared
            .trace("drain_started", session, 0, timeout.as_millis() as u64);
        let shared = Arc::clone(&self.shared);
        runtime.spawn(async move {
            shared
                .events
                .send_reliable(Event::simple(EventKind::Draining, session))
                .await;
            tokio::time::sleep(timeout).await;
            connection.close(VarInt::from_u32(code), &reason);
            shared.trace("drain_deadline_reached", session, 0, code as u64);
        });
        Ok(())
    }

    pub fn set_trace_enabled(&self, enabled: bool) {
        self.shared.trace_enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn session_diagnostics(
        &self,
        session: SessionHandle,
    ) -> Result<SessionDiagnostics, ClientError> {
        let state = self
            .shared
            .state(session)
            .ok_or(ClientError::InvalidSession)?;
        let connection = self.shared.session(session);
        Ok(SessionDiagnostics {
            state,
            stable_id: connection
                .as_ref()
                .map_or(0, |value| value.stable_id() as u64),
            rtt_micros: connection
                .as_ref()
                .map_or(0, |value| value.rtt().as_micros() as u64),
            max_datagram_size: connection
                .and_then(|value| value.max_datagram_size())
                .unwrap_or(0) as u64,
        })
    }

    pub fn poll(&self) -> Option<Event> {
        self.shared.events.poll()
    }

    pub fn stats(&self) -> ClientStats {
        ClientStats {
            dropped_datagrams: self.shared.events.dropped_datagrams.load(Ordering::Relaxed),
            queued_events: self.shared.events.queued.load(Ordering::Relaxed),
            active_sessions: self
                .shared
                .sessions
                .lock()
                .map(|sessions| sessions.len() as u64)
                .unwrap_or(0),
            active_streams: self
                .shared
                .send_streams
                .lock()
                .map(|streams| streams.len() as u64)
                .unwrap_or(0),
            active_draining_sessions: self
                .shared
                .session_states
                .lock()
                .map(|states| {
                    states
                        .values()
                        .filter(|state| **state == SessionState::Draining)
                        .count() as u64
                })
                .unwrap_or(0),
            datagrams_sent: self.shared.datagrams_sent.load(Ordering::Relaxed),
            datagrams_received: self.shared.datagrams_received.load(Ordering::Relaxed),
            stream_bytes_sent: self.shared.stream_bytes_sent.load(Ordering::Relaxed),
            stream_bytes_received: self.shared.stream_bytes_received.load(Ordering::Relaxed),
            connection_failures: self.shared.connection_failures.load(Ordering::Relaxed),
            dropped_trace_events: self
                .shared
                .events
                .dropped_trace_events
                .load(Ordering::Relaxed),
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Ok(sessions) = self.shared.sessions.lock() {
            for connection in sessions.values() {
                connection.close(VarInt::from_u32(0), b"client shutdown");
            }
        }
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

async fn connect_task(
    shared: Arc<Shared>,
    session: SessionHandle,
    url: String,
    options: ConnectOptions,
) -> Result<(), TransportError> {
    let builder = ClientConfig::builder().with_bind_default();
    let config = match options.certificate_mode {
        CertificateMode::NativeRoots => builder.with_native_certs().build(),
        CertificateMode::ServerCertificateHashes(hashes) => builder
            .with_server_certificate_hashes(hashes.into_iter().map(Sha256Digest::new))
            .build(),
        CertificateMode::CustomCaCertificates(certificates) => {
            let mut roots = rustls::RootCertStore::empty();
            for certificate in certificates {
                roots
                    .add(rustls::pki_types::CertificateDer::from(certificate))
                    .map_err(|error| {
                        TransportError::invalid(format!("invalid custom CA certificate: {error}"))
                    })?;
            }
            if roots.is_empty() {
                return Err(TransportError::invalid("custom CA list is empty"));
            }
            let tls = wtransport::tls::client::build_default_tls_config(Arc::new(roots), None);
            builder.with_custom_tls(tls).build()
        }
        #[cfg(feature = "dangerous-insecure")]
        CertificateMode::Insecure => builder.with_no_cert_validation().build(),
    };
    let endpoint = Endpoint::client(config).map_err(TransportError::from_network)?;
    let connection = tokio::time::timeout(options.connect_timeout, endpoint.connect(url))
        .await
        .map_err(|_| TransportError::from_network("connection timeout"))?
        .map_err(TransportError::from_network)?;
    let connection = Arc::new(connection);
    shared
        .sessions
        .lock()
        .map_err(|_| TransportError::invalid("session lock poisoned"))?
        .insert(session, Arc::clone(&connection));
    shared
        .session_states
        .lock()
        .map_err(|_| TransportError::invalid("session state lock poisoned"))?
        .insert(session, SessionState::Connected);
    shared.trace("connected", session, 0, connection.rtt().as_micros() as u64);
    shared
        .events
        .send_reliable(Event::simple(EventKind::Connected, session))
        .await;

    receive_connection(shared, session, connection).await;
    drop(endpoint);
    Ok(())
}

async fn receive_connection(
    shared: Arc<Shared>,
    session: SessionHandle,
    connection: Arc<Connection>,
) {
    loop {
        tokio::select! {
            datagram = connection.receive_datagram() => match datagram {
                Ok(data) => {
                    shared.datagrams_received.fetch_add(1, Ordering::Relaxed);
                    shared.trace("datagram_received", session, 0, data.len() as u64);
                    shared.events.send_datagram(Event {
                        kind: EventKind::Datagram,
                        session,
                        stream: 0,
                        code: 0,
                        data: data.to_vec(),
                        error: None,
                    });
                },
                Err(error) => {
                    send_closed(&shared, session, error).await;
                    break;
                }
            },
            incoming = connection.accept_bi() => match incoming {
                Ok((send, recv)) => {
                    let stream = shared.next_stream();
                    if let Ok(mut streams) = shared.send_streams.lock() {
                        streams.insert(stream, Arc::new(AsyncMutex::new(send)));
                    }
                    shared.events.send_reliable(Event {
                        kind: EventKind::IncomingBidirectionalStream,
                        session,
                        stream,
                        code: 0,
                        data: Vec::new(),
                        error: None,
                    }).await;
                    spawn_receive_stream(Arc::clone(&shared), session, stream, recv);
                }
                Err(error) => {
                    send_closed(&shared, session, error).await;
                    break;
                }
            },
            incoming = connection.accept_uni() => match incoming {
                Ok(recv) => {
                    let stream = shared.next_stream();
                    shared.events.send_reliable(Event {
                        kind: EventKind::IncomingUnidirectionalStream,
                        session,
                        stream,
                        code: 0,
                        data: Vec::new(),
                        error: None,
                    }).await;
                    spawn_receive_stream(Arc::clone(&shared), session, stream, recv);
                }
                Err(error) => {
                    send_closed(&shared, session, error).await;
                    break;
                }
            }
        }
    }
}

fn spawn_receive_stream(
    shared: Arc<Shared>,
    session: SessionHandle,
    stream: StreamHandle,
    mut recv: wtransport::RecvStream,
) {
    tokio::spawn(async move {
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            match recv.read(&mut buffer).await {
                Ok(Some(size)) => {
                    shared
                        .stream_bytes_received
                        .fetch_add(size as u64, Ordering::Relaxed);
                    shared.trace("stream_data_received", session, stream, size as u64);
                    shared
                        .events
                        .send_reliable(Event {
                            kind: EventKind::StreamData,
                            session,
                            stream,
                            code: 0,
                            data: buffer[..size].to_vec(),
                            error: None,
                        })
                        .await;
                }
                Ok(None) => {
                    shared
                        .events
                        .send_reliable(Event {
                            kind: EventKind::StreamFinished,
                            session,
                            stream,
                            code: 0,
                            data: Vec::new(),
                            error: None,
                        })
                        .await;
                    break;
                }
                Err(error) => {
                    send_stream_error(&shared, session, stream, error).await;
                    break;
                }
            }
        }
    });
}

async fn send_stream_error(
    shared: &Shared,
    session: SessionHandle,
    stream: StreamHandle,
    error: impl std::fmt::Display,
) {
    let error = TransportError::from_network(error);
    shared
        .events
        .send_reliable(Event {
            kind: EventKind::StreamReset,
            session,
            stream,
            code: error.code,
            data: error.message.as_bytes().to_vec(),
            error: Some(error),
        })
        .await;
}

async fn send_closed(shared: &Shared, session: SessionHandle, error: impl std::fmt::Display) {
    let error = TransportError::from_network(error);
    shared.trace("session_closed", session, 0, 0);
    shared
        .events
        .send_reliable(Event {
            kind: EventKind::Closed,
            session,
            stream: 0,
            code: error.code,
            data: error.message.as_bytes().to_vec(),
            error: Some(error),
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_https_urls() {
        let client = Client::new(8).unwrap();
        assert!(matches!(
            client.connect("http://localhost:4433", ConnectOptions::default()),
            Err(ClientError::InvalidArgument(_))
        ));
    }

    #[test]
    fn handles_are_monotonic_across_failed_connections() {
        let client = Client::new(8).unwrap();
        let first = client
            .connect("https://127.0.0.1:1", ConnectOptions::default())
            .unwrap();
        let second = client
            .connect("https://127.0.0.1:2", ConnectOptions::default())
            .unwrap();
        assert!(second > first);
    }

    #[test]
    fn repeated_create_and_drop_is_safe() {
        for _ in 0..100 {
            let client = Client::new(2).unwrap();
            assert_eq!(client.stats(), ClientStats::default());
        }
    }

    #[tokio::test]
    async fn reliable_queue_applies_backpressure_and_datagrams_drop() {
        let queue = Arc::new(EventQueue::new(1));
        queue.send_datagram(Event::simple(EventKind::Datagram, 1));
        queue.send_datagram(Event::simple(EventKind::Datagram, 1));
        assert_eq!(queue.dropped_datagrams.load(Ordering::Relaxed), 1);

        let cloned = Arc::clone(&queue);
        let blocked = tokio::spawn(async move {
            cloned
                .send_reliable(Event::simple(EventKind::Connected, 1))
                .await;
        });
        tokio::task::yield_now().await;
        assert!(!blocked.is_finished());
        assert!(queue.poll().is_some());
        blocked.await.unwrap();
        assert_eq!(queue.poll().unwrap().kind, EventKind::Connected);
    }

    #[test]
    fn trace_events_are_opt_in_bounded_and_metadata_only() {
        let queue = EventQueue::new(1);
        queue.send_trace(false, "disabled", 1, 2, 3);
        assert!(queue.poll().is_none());

        queue.send_trace(true, "stream_write_completed", 7, 9, 128);
        queue.send_trace(true, "must_drop", 7, 9, 256);
        assert_eq!(queue.dropped_trace_events.load(Ordering::Relaxed), 1);
        let event = queue.poll().unwrap();
        assert_eq!(event.kind, EventKind::Trace);
        assert_eq!(event.session, 7);
        assert_eq!(event.stream, 9);
        assert_eq!(event.code, 128);
        assert_eq!(event.data, b"stream_write_completed");
        assert!(event.error.is_none());
    }
}
