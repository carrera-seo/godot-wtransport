extends SceneTree

var client: WebTransportClient
var sessions: Array[WebTransportSession] = []
var echoes := 0

func _initialize() -> void:
    client = WebTransportClient.new()
    root.add_child(client)
    client.connection_succeeded.connect(_on_connected)
    client.connection_failed.connect(_on_connection_failed)
    var options := WebTransportTlsOptions.new()
    options.custom_ca_pem = FileAccess.get_file_as_bytes(OS.get_environment("GWT_TEST_CUSTOM_CA"))
    for _index in range(2):
        assert(client.connect_to_url(OS.get_environment("GWT_TEST_URL"), options) != 0)
    create_timer(15.0).timeout.connect(_on_timeout)

func _on_connected(session: WebTransportSession) -> void:
    sessions.append(session)
    session.datagram_received.connect(_on_datagram_received.bind(session))
    assert(session.send_datagram(("session-%d" % sessions.size()).to_utf8_buffer()) == OK)

func _on_datagram_received(data: PackedByteArray, session: WebTransportSession) -> void:
    assert(data.get_string_from_utf8().begins_with("session-"))
    echoes += 1
    if echoes != 2:
        return
    assert(client.get_connection_stats().active_sessions == 2)
    var first := sessions[0].get_diagnostics()
    var second := sessions[1].get_diagnostics()
    assert(first.stable_id != second.stable_id)
    for active_session in sessions:
        active_session.close(0, "multi-session complete")
    print("godot-wtransport concurrent sessions passed")
    quit(0)

func _on_connection_failed(error: Dictionary) -> void:
    push_error("Multi-session connection failed: %s" % error)
    quit(2)

func _on_timeout() -> void:
    push_error("Multi-session test timed out")
    quit(3)
