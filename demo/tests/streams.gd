extends SceneTree

var client: WebTransportClient
var session: WebTransportSession
var bidi_stream: WebTransportStream
var uni_stream: WebTransportStream
var incoming_uni_stream: WebTransportStream
var bidi_received := false
var uni_received := false

func _initialize() -> void:
    client = WebTransportClient.new()
    root.add_child(client)
    client.connection_succeeded.connect(_on_connected)
    client.connection_failed.connect(_on_connection_failed)
    var options := WebTransportTlsOptions.new()
    options.custom_ca_pem = FileAccess.get_file_as_bytes(OS.get_environment("GWT_TEST_CUSTOM_CA"))
    assert(client.connect_to_url(OS.get_environment("GWT_TEST_URL"), options) != 0)
    create_timer(15.0).timeout.connect(_on_timeout)

func _on_connected(connected_session: WebTransportSession) -> void:
    session = connected_session
    session.incoming_unidirectional_stream.connect(_on_incoming_unidirectional_stream)

    bidi_stream = session.create_bidirectional_stream()
    assert(bidi_stream != null)
    bidi_stream.data_received.connect(_on_bidi_data)
    bidi_stream.opened.connect(_on_bidi_opened)

func _on_bidi_opened() -> void:
    assert(bidi_stream.write("godot-bidi-echo".to_utf8_buffer()) == OK)
    assert(bidi_stream.finish() == OK)

func _on_uni_opened() -> void:
    assert(uni_stream.write("godot-uni-echo".to_utf8_buffer()) == OK)
    assert(uni_stream.finish() == OK)

func _on_bidi_data(data: PackedByteArray) -> void:
    assert(data.get_string_from_utf8() == "godot-bidi-echo")
    bidi_received = true
    uni_stream = session.create_unidirectional_stream()
    assert(uni_stream != null)
    uni_stream.opened.connect(_on_uni_opened)

func _on_incoming_unidirectional_stream(stream: WebTransportStream) -> void:
    incoming_uni_stream = stream
    incoming_uni_stream.data_received.connect(_on_uni_data)

func _on_uni_data(data: PackedByteArray) -> void:
    assert(data.get_string_from_utf8() == "godot-uni-echo")
    uni_received = true
    _finish_if_complete()

func _finish_if_complete() -> void:
    if bidi_received and uni_received:
        session.closed.connect(_on_session_closed)
        session.close(0, "stream smoke complete")

func _on_session_closed(_close_info: Dictionary) -> void:
    bidi_stream = null
    uni_stream = null
    incoming_uni_stream = null
    session = null
    client.queue_free()
    await process_frame
    client = null
    print("godot-wtransport native stream round trip passed")
    quit(0)

func _on_connection_failed(error: Dictionary) -> void:
    push_error("Stream test connection failed: %s" % error)
    quit(2)

func _on_timeout() -> void:
    push_error("Stream test timed out")
    quit(3)
