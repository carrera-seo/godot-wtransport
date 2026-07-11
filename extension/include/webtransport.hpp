#ifndef GODOT_WTRANSPORT_CLASSES_HPP
#define GODOT_WTRANSPORT_CLASSES_HPP

#include <cstdint>
#include <unordered_map>

#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/classes/ref.hpp>
#include <godot_cpp/variant/array.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/variant/packed_byte_array.hpp>
#include <godot_cpp/variant/string.hpp>

#include "godot_wtransport.h"

namespace godot {

class WebTransportClient;

class WebTransportTlsOptions : public RefCounted {
    GDCLASS(WebTransportTlsOptions, RefCounted)
    Array server_certificate_hashes;
    PackedByteArray custom_ca_pem;

protected:
    static void _bind_methods();

public:
    void set_server_certificate_hashes(const Array &p_hashes);
    Array get_server_certificate_hashes() const;
    void add_server_certificate_hash(const PackedByteArray &p_hash);
    void set_custom_ca_pem(const PackedByteArray &p_pem);
    PackedByteArray get_custom_ca_pem() const;
};

class WebTransportStream : public RefCounted {
    GDCLASS(WebTransportStream, RefCounted)
    ObjectID client_id;
    uint64_t handle = 0;
    bool writable = false;

protected:
    static void _bind_methods();

public:
    void configure(ObjectID p_client_id, uint64_t p_handle, bool p_writable);
    int64_t get_handle() const;
    bool is_writable() const;
    Error write(const PackedByteArray &p_data);
    Error finish();
};

class WebTransportSession : public RefCounted {
    GDCLASS(WebTransportSession, RefCounted)
    ObjectID client_id;
    uint64_t handle = 0;

protected:
    static void _bind_methods();

public:
    void configure(ObjectID p_client_id, uint64_t p_handle);
    int64_t get_handle() const;
    Error send_datagram(const PackedByteArray &p_data);
    Ref<WebTransportStream> create_bidirectional_stream();
    Ref<WebTransportStream> create_unidirectional_stream();
    Error close(int64_t p_code = 0, const String &p_reason = String());
};

class WebTransportClient : public Node {
    GDCLASS(WebTransportClient, Node)
    GwtClient *client = nullptr;
    std::unordered_map<uint64_t, Ref<WebTransportSession>> sessions;
    std::unordered_map<uint64_t, Ref<WebTransportStream>> streams;

    static PackedByteArray copy_bytes(const uint8_t *p_data, size_t p_size);
    Dictionary error_dictionary(const GwtEvent &p_event) const;
    Ref<WebTransportSession> session_for(uint64_t p_handle);
    Ref<WebTransportStream> stream_for(uint64_t p_handle, bool p_writable);
    void dispatch_event(GwtEvent &p_event);

protected:
    static void _bind_methods();

public:
    WebTransportClient();
    ~WebTransportClient() override;
    void _process(double p_delta) override;
    int64_t connect_to_url(const String &p_url, const Ref<WebTransportTlsOptions> &p_tls_options = Ref<WebTransportTlsOptions>());
#ifdef GWT_ENABLE_INSECURE
    int64_t connect_insecure_for_testing(const String &p_url);
#endif
    Error send_datagram(uint64_t p_session, const PackedByteArray &p_data);
    Ref<WebTransportStream> open_bidirectional_stream(uint64_t p_session);
    Ref<WebTransportStream> open_unidirectional_stream(uint64_t p_session);
    Error write_stream(uint64_t p_stream, const PackedByteArray &p_data);
    Error finish_stream(uint64_t p_stream);
    Error close_session(uint64_t p_session, uint32_t p_code, const PackedByteArray &p_reason);
    Dictionary get_connection_stats() const;
};

} // namespace godot

#endif
