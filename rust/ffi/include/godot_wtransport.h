#ifndef GODOT_WTRANSPORT_H
#define GODOT_WTRANSPORT_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GWT_ABI_VERSION 1u

typedef struct GwtClient GwtClient;

typedef enum GwtStatus {
    GWT_STATUS_OK = 0,
    GWT_STATUS_NO_EVENT = 1,
    GWT_STATUS_INVALID_ARGUMENT = -1,
    GWT_STATUS_INVALID_HANDLE = -2,
    GWT_STATUS_INTERNAL_ERROR = -3,
    GWT_STATUS_PANIC = -4
} GwtStatus;

typedef enum GwtEventKind {
    GWT_EVENT_CONNECTED = 1,
    GWT_EVENT_CONNECTION_FAILED = 2,
    GWT_EVENT_CLOSED = 3,
    GWT_EVENT_DATAGRAM = 4,
    GWT_EVENT_STREAM_OPENED = 5,
    GWT_EVENT_INCOMING_BIDIRECTIONAL_STREAM = 6,
    GWT_EVENT_INCOMING_UNIDIRECTIONAL_STREAM = 7,
    GWT_EVENT_STREAM_DATA = 8,
    GWT_EVENT_STREAM_FINISHED = 9,
    GWT_EVENT_STREAM_RESET = 10,
    GWT_EVENT_ERROR = 11
} GwtEventKind;

typedef struct GwtEvent {
    uint32_t kind;
    uint32_t error_category;
    uint64_t session;
    uint64_t stream;
    int64_t code;
    bool retryable;
    uint64_t transport_error_code;
    uint64_t http3_error_code;
    uint64_t stream_error_code;
    uint8_t tls_alert;
    int32_t os_error;
    uint8_t *data;
    size_t data_len;
} GwtEvent;

typedef struct GwtClientStats {
    uint64_t dropped_datagrams;
    uint64_t queued_events;
    uint64_t active_sessions;
    uint64_t active_streams;
} GwtClientStats;

uint32_t gwt_abi_version(void);
GwtClient *gwt_client_create(size_t event_capacity);
void gwt_client_destroy(GwtClient *client);
GwtStatus gwt_client_connect(GwtClient *client, const char *url, uint64_t *out_session);
GwtStatus gwt_client_connect_hashes(GwtClient *client, const char *url,
                                    const uint8_t *hashes, size_t hash_count,
                                    uint64_t *out_session);
GwtStatus gwt_client_send_datagram(GwtClient *client, uint64_t session,
                                   const uint8_t *data, size_t data_len);
GwtStatus gwt_client_open_bidirectional_stream(GwtClient *client, uint64_t session,
                                                uint64_t *out_stream);
GwtStatus gwt_client_open_unidirectional_stream(GwtClient *client, uint64_t session,
                                                 uint64_t *out_stream);
GwtStatus gwt_client_write_stream(GwtClient *client, uint64_t stream,
                                  const uint8_t *data, size_t data_len);
GwtStatus gwt_client_finish_stream(GwtClient *client, uint64_t stream);
GwtStatus gwt_client_close(GwtClient *client, uint64_t session, uint32_t code,
                           const uint8_t *reason, size_t reason_len);
GwtStatus gwt_client_poll(GwtClient *client, GwtEvent *out_event);
GwtStatus gwt_client_stats(GwtClient *client, GwtClientStats *out_stats);
void gwt_event_free(GwtEvent *event);

#ifdef __cplusplus
}
#endif

#endif
