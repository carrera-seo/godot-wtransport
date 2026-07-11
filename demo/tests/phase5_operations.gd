extends SceneTree

var client: WebTransportClient
var session: WebTransportSession
var trace_names: Array[String] = []
var draining_seen := false

func _initialize() -> void:
    client = WebTransportClient.new()
    root.add_child(client)
    client.trace_enabled = true
    client.trace_event.connect(_on_trace_event)
    client.connection_succeeded.connect(_on_connected)
    client.connection_failed.connect(_on_connection_failed)
    var options := WebTransportTlsOptions.new()
    options.custom_ca_pem = FileAccess.get_file_as_bytes(OS.get_environment("GWT_TEST_CUSTOM_CA"))
    assert(client.connect_to_url(OS.get_environment("GWT_TEST_URL"), options) != 0)
    create_timer(15.0).timeout.connect(_on_timeout)

func _on_connected(connected_session: WebTransportSession) -> void:
    session = connected_session
    session.datagram_received.connect(_on_datagram_received)
    session.draining_started.connect(_on_draining_started)
    session.closed.connect(_on_closed)
    var diagnostics := session.get_diagnostics()
    assert(diagnostics.state == 2)
    assert(diagnostics.stable_id > 0)
    assert(diagnostics.max_datagram_size > 0)
    assert(session.send_datagram("phase5-stats".to_utf8_buffer()) == OK)

func _on_datagram_received(data: PackedByteArray) -> void:
    assert(data.get_string_from_utf8() == "phase5-stats")
    var stats := client.get_connection_stats()
    assert(stats.datagrams_sent >= 1)
    assert(stats.datagrams_received >= 1)
    assert(stats.active_sessions == 1)
    assert(session.drain(100, 7, "phase5 drain") == OK)
    assert(session.get_diagnostics().state == 3)
    assert(session.send_datagram(PackedByteArray([1])) != OK)
    assert(session.create_unidirectional_stream() == null)

func _on_draining_started() -> void:
    draining_seen = true

func _on_closed(_close_info: Dictionary) -> void:
    assert(draining_seen)
    assert("connect_started" in trace_names)
    assert("connected" in trace_names)
    assert("datagram_sent" in trace_names)
    assert("datagram_received" in trace_names)
    assert("drain_started" in trace_names)
    print("godot-wtransport phase 5 operations passed")
    quit(0)

func _on_trace_event(event: Dictionary) -> void:
    trace_names.append(event.name)
    assert(not event.has("payload"))
    assert(not event.has("url"))

func _on_connection_failed(error: Dictionary) -> void:
    push_error("Phase 5 connection failed: %s" % error)
    quit(2)

func _on_timeout() -> void:
    push_error("Phase 5 operations timed out")
    quit(3)
