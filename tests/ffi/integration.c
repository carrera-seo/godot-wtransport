#include "godot_wtransport.h"

#include <assert.h>
#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static int hex_value(char value) {
    if (value >= '0' && value <= '9') return value - '0';
    value = (char)tolower((unsigned char)value);
    if (value >= 'a' && value <= 'f') return value - 'a' + 10;
    return -1;
}

static int parse_hash(const char *input, uint8_t output[32]) {
    size_t index = 0;
    while (*input != '\0' && index < 32) {
        while (*input == ':' || *input == ' ') ++input;
        int high = hex_value(*input++);
        int low = hex_value(*input++);
        if (high < 0 || low < 0) return 0;
        output[index++] = (uint8_t)((high << 4) | low);
    }
    return index == 32;
}

static void wait_briefly(void) {
    struct timespec duration = {.tv_sec = 0, .tv_nsec = 10000000};
    nanosleep(&duration, NULL);
}

static int wait_for_event(GwtClient *client, uint32_t kind, uint64_t stream,
                          const uint8_t *payload, size_t payload_len) {
    for (int attempt = 0; attempt < 1500; ++attempt) {
        GwtEvent event = {0};
        GwtStatus status = gwt_client_poll(client, &event);
        if (status == GWT_STATUS_NO_EVENT) {
            wait_briefly();
            continue;
        }
        assert(status == GWT_STATUS_OK);
        if (event.kind == GWT_EVENT_CONNECTION_FAILED || event.kind == GWT_EVENT_ERROR) {
            fprintf(stderr, "transport error: %.*s\n", (int)event.data_len, event.data);
            gwt_event_free(&event);
            return 0;
        }
        int matches = event.kind == kind && (stream == 0 || event.stream == stream);
        if (matches && payload != NULL) {
            matches = event.data_len == payload_len &&
                      memcmp(event.data, payload, payload_len) == 0;
        }
        gwt_event_free(&event);
        if (matches) return 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: %s URL CERTIFICATE_HASH\n", argv[0]);
        return 2;
    }
    uint8_t hash[32];
    assert(parse_hash(argv[2], hash));
    GwtClient *client = gwt_client_create(64);
    assert(client != NULL);

    uint64_t session = 0;
    assert(gwt_client_connect_hashes(client, argv[1], hash, 1, &session) == GWT_STATUS_OK);

    assert(wait_for_event(client, GWT_EVENT_CONNECTED, 0, NULL, 0));

    static const uint8_t payload[] = "phase-1-ffi";
    assert(gwt_client_send_datagram(client, session, payload, sizeof(payload) - 1) == GWT_STATUS_OK);

    assert(wait_for_event(client, GWT_EVENT_DATAGRAM, 0, payload, sizeof(payload) - 1));

    static const uint8_t stream_payload[] = "phase-1-stream";
    uint64_t bidirectional = 0;
    assert(gwt_client_open_bidirectional_stream(client, session, &bidirectional) == GWT_STATUS_OK);
    assert(wait_for_event(client, GWT_EVENT_STREAM_OPENED, bidirectional, NULL, 0));
    assert(gwt_client_write_stream(client, bidirectional, stream_payload,
                                   sizeof(stream_payload) - 1) == GWT_STATUS_OK);
    assert(gwt_client_finish_stream(client, bidirectional) == GWT_STATUS_OK);
    assert(wait_for_event(client, GWT_EVENT_STREAM_DATA, bidirectional, stream_payload,
                          sizeof(stream_payload) - 1));

    uint64_t unidirectional = 0;
    assert(gwt_client_open_unidirectional_stream(client, session, &unidirectional) == GWT_STATUS_OK);
    assert(wait_for_event(client, GWT_EVENT_STREAM_OPENED, unidirectional, NULL, 0));
    assert(gwt_client_write_stream(client, unidirectional, stream_payload,
                                   sizeof(stream_payload) - 1) == GWT_STATUS_OK);
    assert(gwt_client_finish_stream(client, unidirectional) == GWT_STATUS_OK);
    assert(wait_for_event(client, GWT_EVENT_INCOMING_UNIDIRECTIONAL_STREAM, 0, NULL, 0));
    assert(wait_for_event(client, GWT_EVENT_STREAM_DATA, 0, stream_payload,
                          sizeof(stream_payload) - 1));
    assert(gwt_client_close(client, session, 0, NULL, 0) == GWT_STATUS_OK);
    assert(wait_for_event(client, GWT_EVENT_CLOSED, 0, NULL, 0));

    for (int iteration = 0; iteration < 20; ++iteration) {
        uint64_t repeated_session = 0;
        assert(gwt_client_connect_hashes(client, argv[1], hash, 1, &repeated_session) ==
               GWT_STATUS_OK);
        assert(wait_for_event(client, GWT_EVENT_CONNECTED, 0, NULL, 0));
        assert(gwt_client_close(client, repeated_session, 0, NULL, 0) == GWT_STATUS_OK);
        assert(wait_for_event(client, GWT_EVENT_CLOSED, 0, NULL, 0));
    }

    gwt_client_destroy(client);
    return 0;
}
