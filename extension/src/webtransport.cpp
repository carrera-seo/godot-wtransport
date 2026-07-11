#include "webtransport.hpp"

#include <cstring>
#include <vector>

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/core/error_macros.hpp>
#include <godot_cpp/core/object.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

namespace godot {

static WebTransportClient *resolve_client(ObjectID p_id) {
    return Object::cast_to<WebTransportClient>(ObjectDB::get_instance(p_id));
}

void WebTransportTlsOptions::_bind_methods() {
    ClassDB::bind_method(D_METHOD("set_server_certificate_hashes", "hashes"), &WebTransportTlsOptions::set_server_certificate_hashes);
    ClassDB::bind_method(D_METHOD("get_server_certificate_hashes"), &WebTransportTlsOptions::get_server_certificate_hashes);
    ClassDB::bind_method(D_METHOD("add_server_certificate_hash", "hash"), &WebTransportTlsOptions::add_server_certificate_hash);
    ClassDB::bind_method(D_METHOD("set_custom_ca_pem", "pem"), &WebTransportTlsOptions::set_custom_ca_pem);
    ClassDB::bind_method(D_METHOD("get_custom_ca_pem"), &WebTransportTlsOptions::get_custom_ca_pem);
    ADD_PROPERTY(PropertyInfo(Variant::ARRAY, "server_certificate_hashes", PROPERTY_HINT_ARRAY_TYPE, "PackedByteArray"), "set_server_certificate_hashes", "get_server_certificate_hashes");
    ADD_PROPERTY(PropertyInfo(Variant::PACKED_BYTE_ARRAY, "custom_ca_pem"), "set_custom_ca_pem", "get_custom_ca_pem");
}

void WebTransportTlsOptions::set_server_certificate_hashes(const Array &p_hashes) {
    for (int64_t index = 0; index < p_hashes.size(); ++index) {
        PackedByteArray hash = p_hashes[index];
        ERR_FAIL_COND_MSG(hash.size() != 32, "Each SHA-256 certificate hash must contain exactly 32 bytes.");
    }
    server_certificate_hashes = p_hashes;
}

Array WebTransportTlsOptions::get_server_certificate_hashes() const {
    return server_certificate_hashes;
}

void WebTransportTlsOptions::add_server_certificate_hash(const PackedByteArray &p_hash) {
    ERR_FAIL_COND_MSG(p_hash.size() != 32, "A SHA-256 certificate hash must contain exactly 32 bytes.");
    server_certificate_hashes.push_back(p_hash);
}

void WebTransportTlsOptions::set_custom_ca_pem(const PackedByteArray &p_pem) {
    custom_ca_pem = p_pem;
}

PackedByteArray WebTransportTlsOptions::get_custom_ca_pem() const {
    return custom_ca_pem;
}

void WebTransportStream::_bind_methods() {
    ClassDB::bind_method(D_METHOD("get_handle"), &WebTransportStream::get_handle);
    ClassDB::bind_method(D_METHOD("is_writable"), &WebTransportStream::is_writable);
    ClassDB::bind_method(D_METHOD("write", "data"), &WebTransportStream::write);
    ClassDB::bind_method(D_METHOD("finish"), &WebTransportStream::finish);
    ADD_SIGNAL(MethodInfo("data_received", PropertyInfo(Variant::PACKED_BYTE_ARRAY, "data")));
    ADD_SIGNAL(MethodInfo("finished"));
    ADD_SIGNAL(MethodInfo("reset", PropertyInfo(Variant::INT, "error_code")));
}

void WebTransportStream::configure(ObjectID p_client_id, uint64_t p_handle, bool p_writable) {
    client_id = p_client_id;
    handle = p_handle;
    writable = p_writable;
}

int64_t WebTransportStream::get_handle() const {
    return static_cast<int64_t>(handle);
}

bool WebTransportStream::is_writable() const {
    return writable;
}

Error WebTransportStream::write(const PackedByteArray &p_data) {
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, ERR_UNCONFIGURED);
    ERR_FAIL_COND_V_MSG(!writable, ERR_UNAVAILABLE, "The stream is receive-only or already finished.");
    return owner->write_stream(handle, p_data);
}

Error WebTransportStream::finish() {
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, ERR_UNCONFIGURED);
    ERR_FAIL_COND_V_MSG(!writable, ERR_UNAVAILABLE, "The stream is receive-only or already finished.");
    Error result = owner->finish_stream(handle);
    if (result == OK) {
        writable = false;
    }
    return result;
}

void WebTransportSession::_bind_methods() {
    ClassDB::bind_method(D_METHOD("get_handle"), &WebTransportSession::get_handle);
    ClassDB::bind_method(D_METHOD("send_datagram", "data"), &WebTransportSession::send_datagram);
    ClassDB::bind_method(D_METHOD("create_bidirectional_stream"), &WebTransportSession::create_bidirectional_stream);
    ClassDB::bind_method(D_METHOD("create_unidirectional_stream"), &WebTransportSession::create_unidirectional_stream);
    ClassDB::bind_method(D_METHOD("get_diagnostics"), &WebTransportSession::get_diagnostics);
    ClassDB::bind_method(D_METHOD("drain", "timeout_ms", "code", "reason"), &WebTransportSession::drain, DEFVAL(0), DEFVAL(String()));
    ClassDB::bind_method(D_METHOD("close", "code", "reason"), &WebTransportSession::close, DEFVAL(0), DEFVAL(String()));
    ADD_SIGNAL(MethodInfo("draining_started"));
    ADD_SIGNAL(MethodInfo("closed", PropertyInfo(Variant::DICTIONARY, "close_info")));
    ADD_SIGNAL(MethodInfo("datagram_received", PropertyInfo(Variant::PACKED_BYTE_ARRAY, "data")));
    ADD_SIGNAL(MethodInfo("incoming_bidirectional_stream", PropertyInfo(Variant::OBJECT, "stream", PROPERTY_HINT_RESOURCE_TYPE, "WebTransportStream")));
    ADD_SIGNAL(MethodInfo("incoming_unidirectional_stream", PropertyInfo(Variant::OBJECT, "stream", PROPERTY_HINT_RESOURCE_TYPE, "WebTransportStream")));
}

void WebTransportSession::configure(ObjectID p_client_id, uint64_t p_handle) {
    client_id = p_client_id;
    handle = p_handle;
}

int64_t WebTransportSession::get_handle() const {
    return static_cast<int64_t>(handle);
}

Error WebTransportSession::send_datagram(const PackedByteArray &p_data) {
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, ERR_UNCONFIGURED);
    return owner->send_datagram(handle, p_data);
}

Ref<WebTransportStream> WebTransportSession::create_bidirectional_stream() {
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, Ref<WebTransportStream>());
    return owner->open_bidirectional_stream(handle);
}

Ref<WebTransportStream> WebTransportSession::create_unidirectional_stream() {
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, Ref<WebTransportStream>());
    return owner->open_unidirectional_stream(handle);
}

Dictionary WebTransportSession::get_diagnostics() const {
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, Dictionary());
    return owner->get_session_diagnostics(handle);
}

Error WebTransportSession::drain(int64_t p_timeout_ms, int64_t p_code, const String &p_reason) {
    ERR_FAIL_COND_V_MSG(p_timeout_ms <= 0, ERR_INVALID_PARAMETER, "Drain timeout must be positive.");
    ERR_FAIL_COND_V_MSG(p_code < 0 || p_code > UINT32_MAX, ERR_INVALID_PARAMETER, "Close code must fit in an unsigned 32-bit integer.");
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, ERR_UNCONFIGURED);
    CharString reason = p_reason.utf8();
    PackedByteArray bytes;
    bytes.resize(reason.length());
    if (reason.length() > 0) {
        std::memcpy(bytes.ptrw(), reason.get_data(), reason.length());
    }
    return owner->drain_session(handle, static_cast<uint64_t>(p_timeout_ms), static_cast<uint32_t>(p_code), bytes);
}

Error WebTransportSession::close(int64_t p_code, const String &p_reason) {
    ERR_FAIL_COND_V_MSG(p_code < 0 || p_code > UINT32_MAX, ERR_INVALID_PARAMETER, "Close code must fit in an unsigned 32-bit integer.");
    WebTransportClient *owner = resolve_client(client_id);
    ERR_FAIL_NULL_V(owner, ERR_UNCONFIGURED);
    CharString reason = p_reason.utf8();
    PackedByteArray bytes;
    bytes.resize(reason.length());
    if (reason.length() > 0) {
        std::memcpy(bytes.ptrw(), reason.get_data(), reason.length());
    }
    return owner->close_session(handle, static_cast<uint32_t>(p_code), bytes);
}

WebTransportClient::WebTransportClient() {
    client = gwt_client_create(1024);
    ERR_FAIL_NULL_MSG(client, "Failed to create the WebTransport runtime.");
    set_process(true);
}

WebTransportClient::~WebTransportClient() {
    if (client != nullptr) {
        gwt_client_destroy(client);
        client = nullptr;
    }
    streams.clear();
    sessions.clear();
}

void WebTransportClient::_bind_methods() {
    ClassDB::bind_method(D_METHOD("connect_to_url", "url", "tls_options"), &WebTransportClient::connect_to_url, DEFVAL(Ref<WebTransportTlsOptions>()));
    ClassDB::bind_method(D_METHOD("get_connection_stats"), &WebTransportClient::get_connection_stats);
    ClassDB::bind_method(D_METHOD("set_trace_enabled", "enabled"), &WebTransportClient::set_trace_enabled);
    ClassDB::bind_method(D_METHOD("is_trace_enabled"), &WebTransportClient::is_trace_enabled);
    ADD_PROPERTY(PropertyInfo(Variant::BOOL, "trace_enabled"), "set_trace_enabled", "is_trace_enabled");
#ifdef GWT_ENABLE_INSECURE
    ClassDB::bind_method(D_METHOD("connect_insecure_for_testing", "url"), &WebTransportClient::connect_insecure_for_testing);
#endif
    ADD_SIGNAL(MethodInfo("connection_succeeded", PropertyInfo(Variant::OBJECT, "session", PROPERTY_HINT_RESOURCE_TYPE, "WebTransportSession")));
    ADD_SIGNAL(MethodInfo("connection_failed", PropertyInfo(Variant::DICTIONARY, "error")));
    ADD_SIGNAL(MethodInfo("trace_event", PropertyInfo(Variant::DICTIONARY, "event")));
}

PackedByteArray WebTransportClient::copy_bytes(const uint8_t *p_data, size_t p_size) {
    PackedByteArray result;
    result.resize(static_cast<int64_t>(p_size));
    if (p_size > 0) {
        std::memcpy(result.ptrw(), p_data, p_size);
    }
    return result;
}

Dictionary WebTransportClient::error_dictionary(const GwtEvent &p_event) const {
    Dictionary result;
    result["category"] = static_cast<int64_t>(p_event.error_category);
    result["code"] = p_event.code;
    result["message"] = String::utf8(reinterpret_cast<const char *>(p_event.data), static_cast<int64_t>(p_event.data_len));
    result["retryable"] = p_event.retryable;
    result["transport_error_code"] = static_cast<int64_t>(p_event.transport_error_code);
    result["http3_error_code"] = static_cast<int64_t>(p_event.http3_error_code);
    result["stream_error_code"] = static_cast<int64_t>(p_event.stream_error_code);
    result["tls_alert"] = static_cast<int64_t>(p_event.tls_alert);
    result["os_error"] = static_cast<int64_t>(p_event.os_error);
    return result;
}

Ref<WebTransportSession> WebTransportClient::session_for(uint64_t p_handle) {
    auto found = sessions.find(p_handle);
    if (found != sessions.end()) {
        return found->second;
    }
    Ref<WebTransportSession> session;
    session.instantiate();
    session->configure(ObjectID(get_instance_id()), p_handle);
    sessions.emplace(p_handle, session);
    return session;
}

Ref<WebTransportStream> WebTransportClient::stream_for(uint64_t p_handle, bool p_writable) {
    auto found = streams.find(p_handle);
    if (found != streams.end()) {
        return found->second;
    }
    Ref<WebTransportStream> stream;
    stream.instantiate();
    stream->configure(ObjectID(get_instance_id()), p_handle, p_writable);
    streams.emplace(p_handle, stream);
    return stream;
}

void WebTransportClient::dispatch_event(GwtEvent &p_event) {
    Ref<WebTransportSession> session;
    if (p_event.session != 0) {
        session = session_for(p_event.session);
    }
    switch (p_event.kind) {
        case GWT_EVENT_CONNECTED:
            emit_signal("connection_succeeded", session);
            break;
        case GWT_EVENT_CONNECTION_FAILED:
            emit_signal("connection_failed", error_dictionary(p_event));
            sessions.erase(p_event.session);
            break;
        case GWT_EVENT_CLOSED:
            if (session.is_valid()) {
                session->emit_signal("closed", error_dictionary(p_event));
            }
            sessions.erase(p_event.session);
            break;
        case GWT_EVENT_DATAGRAM:
            if (session.is_valid()) {
                session->emit_signal("datagram_received", copy_bytes(p_event.data, p_event.data_len));
            }
            break;
        case GWT_EVENT_STREAM_OPENED:
            stream_for(p_event.stream, true);
            break;
        case GWT_EVENT_INCOMING_BIDIRECTIONAL_STREAM:
            session->emit_signal("incoming_bidirectional_stream", stream_for(p_event.stream, true));
            break;
        case GWT_EVENT_INCOMING_UNIDIRECTIONAL_STREAM:
            session->emit_signal("incoming_unidirectional_stream", stream_for(p_event.stream, false));
            break;
        case GWT_EVENT_STREAM_DATA:
            stream_for(p_event.stream, false)->emit_signal("data_received", copy_bytes(p_event.data, p_event.data_len));
            break;
        case GWT_EVENT_STREAM_FINISHED:
            stream_for(p_event.stream, false)->emit_signal("finished");
            break;
        case GWT_EVENT_STREAM_RESET:
            stream_for(p_event.stream, false)->emit_signal("reset", p_event.code);
            break;
        case GWT_EVENT_DRAINING:
            if (session.is_valid()) {
                session->emit_signal("draining_started");
            }
            break;
        case GWT_EVENT_TRACE: {
            Dictionary trace;
            trace["name"] = String::utf8(reinterpret_cast<const char *>(p_event.data), static_cast<int64_t>(p_event.data_len));
            trace["session_handle"] = static_cast<int64_t>(p_event.session);
            trace["stream_handle"] = static_cast<int64_t>(p_event.stream);
            trace["value"] = p_event.code;
            emit_signal("trace_event", trace);
            break;
        }
        default:
            UtilityFunctions::push_warning("Unknown WebTransport event kind: ", p_event.kind);
            break;
    }
}

void WebTransportClient::_process(double p_delta) {
    (void)p_delta;
    if (client == nullptr) {
        return;
    }
    for (int processed = 0; processed < 256; ++processed) {
        GwtEvent event = {};
        GwtStatus status = gwt_client_poll(client, &event);
        if (status == GWT_STATUS_NO_EVENT) {
            break;
        }
        if (status != GWT_STATUS_OK) {
            UtilityFunctions::push_error("Failed to poll a WebTransport event.");
            break;
        }
        dispatch_event(event);
        gwt_event_free(&event);
    }
}

int64_t WebTransportClient::connect_to_url(const String &p_url, const Ref<WebTransportTlsOptions> &p_tls_options) {
    ERR_FAIL_NULL_V(client, 0);
    CharString url = p_url.utf8();
    uint64_t session = 0;
    GwtStatus status;
    if (p_tls_options.is_valid() && !p_tls_options->get_custom_ca_pem().is_empty()) {
        PackedByteArray pem = p_tls_options->get_custom_ca_pem();
        status = gwt_client_connect_custom_ca_pem(client, url.get_data(), pem.ptr(), pem.size(), &session);
    } else if (p_tls_options.is_valid() && !p_tls_options->get_server_certificate_hashes().is_empty()) {
        Array hashes = p_tls_options->get_server_certificate_hashes();
        std::vector<uint8_t> raw_hashes;
        raw_hashes.reserve(hashes.size() * 32);
        for (int64_t index = 0; index < hashes.size(); ++index) {
            PackedByteArray hash = hashes[index];
            ERR_FAIL_COND_V_MSG(hash.size() != 32, 0, "Each SHA-256 certificate hash must contain 32 bytes.");
            raw_hashes.insert(raw_hashes.end(), hash.ptr(), hash.ptr() + hash.size());
        }
        status = gwt_client_connect_hashes(client, url.get_data(), raw_hashes.data(), hashes.size(), &session);
    } else {
        status = gwt_client_connect(client, url.get_data(), &session);
    }
    ERR_FAIL_COND_V_MSG(status != GWT_STATUS_OK, 0, "Failed to queue a WebTransport connection.");
    session_for(session);
    return static_cast<int64_t>(session);
}

#ifdef GWT_ENABLE_INSECURE
int64_t WebTransportClient::connect_insecure_for_testing(const String &p_url) {
    ERR_FAIL_NULL_V(client, 0);
    CharString url = p_url.utf8();
    uint64_t session = 0;
    ERR_FAIL_COND_V_MSG(gwt_client_connect_insecure_for_testing(client, url.get_data(), &session) != GWT_STATUS_OK, 0, "Failed to queue an insecure WebTransport connection.");
    session_for(session);
    return static_cast<int64_t>(session);
}
#endif

Error WebTransportClient::send_datagram(uint64_t p_session, const PackedByteArray &p_data) {
    return gwt_client_send_datagram(client, p_session, p_data.ptr(), p_data.size()) == GWT_STATUS_OK ? OK : ERR_CONNECTION_ERROR;
}

Ref<WebTransportStream> WebTransportClient::open_bidirectional_stream(uint64_t p_session) {
    uint64_t handle = 0;
    ERR_FAIL_COND_V(gwt_client_open_bidirectional_stream(client, p_session, &handle) != GWT_STATUS_OK, Ref<WebTransportStream>());
    return stream_for(handle, true);
}

Ref<WebTransportStream> WebTransportClient::open_unidirectional_stream(uint64_t p_session) {
    uint64_t handle = 0;
    ERR_FAIL_COND_V(gwt_client_open_unidirectional_stream(client, p_session, &handle) != GWT_STATUS_OK, Ref<WebTransportStream>());
    return stream_for(handle, true);
}

Error WebTransportClient::write_stream(uint64_t p_stream, const PackedByteArray &p_data) {
    return gwt_client_write_stream(client, p_stream, p_data.ptr(), p_data.size()) == GWT_STATUS_OK ? OK : ERR_CONNECTION_ERROR;
}

Error WebTransportClient::finish_stream(uint64_t p_stream) {
    return gwt_client_finish_stream(client, p_stream) == GWT_STATUS_OK ? OK : ERR_CONNECTION_ERROR;
}

Error WebTransportClient::close_session(uint64_t p_session, uint32_t p_code, const PackedByteArray &p_reason) {
    return gwt_client_close(client, p_session, p_code, p_reason.ptr(), p_reason.size()) == GWT_STATUS_OK ? OK : ERR_CONNECTION_ERROR;
}

Error WebTransportClient::drain_session(uint64_t p_session, uint64_t p_timeout_ms, uint32_t p_code, const PackedByteArray &p_reason) {
    return gwt_client_drain(client, p_session, p_timeout_ms, p_code, p_reason.ptr(), p_reason.size()) == GWT_STATUS_OK ? OK : ERR_CONNECTION_ERROR;
}

Dictionary WebTransportClient::get_session_diagnostics(uint64_t p_session) const {
    Dictionary result;
    GwtSessionDiagnostics diagnostics = {};
    if (client != nullptr && gwt_client_session_diagnostics(client, p_session, &diagnostics) == GWT_STATUS_OK) {
        result["state"] = static_cast<int64_t>(diagnostics.state);
        result["stable_id"] = static_cast<int64_t>(diagnostics.stable_id);
        result["rtt_micros"] = static_cast<int64_t>(diagnostics.rtt_micros);
        result["max_datagram_size"] = static_cast<int64_t>(diagnostics.max_datagram_size);
    }
    return result;
}

void WebTransportClient::set_trace_enabled(bool p_enabled) {
    trace_enabled = p_enabled;
    if (client != nullptr) {
        gwt_client_set_trace_enabled(client, p_enabled);
    }
}

bool WebTransportClient::is_trace_enabled() const {
    return trace_enabled;
}

Dictionary WebTransportClient::get_connection_stats() const {
    Dictionary result;
    GwtClientStats stats = {};
    if (client != nullptr && gwt_client_stats(client, &stats) == GWT_STATUS_OK) {
        result["dropped_datagrams"] = static_cast<int64_t>(stats.dropped_datagrams);
        result["queued_events"] = static_cast<int64_t>(stats.queued_events);
        result["active_sessions"] = static_cast<int64_t>(stats.active_sessions);
        result["active_streams"] = static_cast<int64_t>(stats.active_streams);
        result["active_draining_sessions"] = static_cast<int64_t>(stats.active_draining_sessions);
        result["datagrams_sent"] = static_cast<int64_t>(stats.datagrams_sent);
        result["datagrams_received"] = static_cast<int64_t>(stats.datagrams_received);
        result["stream_bytes_sent"] = static_cast<int64_t>(stats.stream_bytes_sent);
        result["stream_bytes_received"] = static_cast<int64_t>(stats.stream_bytes_received);
        result["connection_failures"] = static_cast<int64_t>(stats.connection_failures);
        result["dropped_trace_events"] = static_cast<int64_t>(stats.dropped_trace_events);
    }
    return result;
}

} // namespace godot
