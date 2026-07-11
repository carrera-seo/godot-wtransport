extends Node

var client: WebTransportClient
var session: WebTransportSession
var timeout_seconds := 10.0

func _ready() -> void:
    client = WebTransportClient.new()
    add_child(client)
    client.connection_succeeded.connect(_on_connection_succeeded)
    client.connection_failed.connect(_on_connection_failed)
    var url := OS.get_environment("GWT_TEST_URL")
    if url.is_empty():
        print("godot-wtransport extension loaded")
        get_tree().quit(0)
        return
    var options := WebTransportTlsOptions.new()
    var hash_text := OS.get_environment("GWT_TEST_CERTIFICATE_HASH")
    if not hash_text.is_empty():
        options.add_server_certificate_hash(_parse_hash(hash_text))
    if client.connect_to_url(url, options) == 0:
        push_error("Failed to queue the WebTransport connection")
        get_tree().quit(2)

func _process(delta: float) -> void:
    timeout_seconds -= delta
    if timeout_seconds <= 0.0:
        push_error("WebTransport smoke test timed out")
        get_tree().quit(3)

func _on_connection_succeeded(connected_session: WebTransportSession) -> void:
    session = connected_session
    session.datagram_received.connect(_on_datagram_received)
    if session.send_datagram("godot-native-echo".to_utf8_buffer()) != OK:
        push_error("Failed to send the smoke-test datagram")
        get_tree().quit(4)

func _on_connection_failed(error: Dictionary) -> void:
    push_error("WebTransport connection failed: %s" % error)
    get_tree().quit(5)

func _on_datagram_received(data: PackedByteArray) -> void:
    if data.get_string_from_utf8() != "godot-native-echo":
        push_error("Unexpected echoed datagram")
        get_tree().quit(6)
        return
    session.close(0, "smoke complete")
    print("godot-wtransport native round trip passed")
    get_tree().quit(0)

func _parse_hash(value: String) -> PackedByteArray:
    var compact := value.replace(":", "")
    var result := PackedByteArray()
    if compact.length() != 64:
        return result
    for index in range(0, compact.length(), 2):
        result.append(compact.substr(index, 2).hex_to_int())
    return result
