extends Node

var client: Node
var session: RefCounted
var timeout_seconds := 10.0

func _ready() -> void:
    client = WebTransportWebClient.new() if OS.has_feature("web") else ClassDB.instantiate("WebTransportClient")
    if client == null:
        push_error("The selected WebTransport backend is unavailable")
        get_tree().quit(1)
        return
    add_child(client)
    client.connection_succeeded.connect(_on_connection_succeeded)
    client.connection_failed.connect(_on_connection_failed)
    var url := OS.get_environment("GWT_TEST_URL")
    if OS.has_feature("web"):
        url = _web_query_parameter("url", url)
    if url.is_empty():
        print("godot-wtransport %s backend loaded" % ("Web" if OS.has_feature("web") else "native"))
        get_tree().quit(0)
        return
    if OS.has_feature("web"):
        var hashes: Array[PackedByteArray] = []
        var web_hash_text := _web_query_parameter("certificate_hash", OS.get_environment("GWT_TEST_CERTIFICATE_HASH"))
        if not web_hash_text.is_empty():
            hashes.append(_parse_hash(web_hash_text))
        if client.connect_to_url(url, hashes) == 0:
            push_error("Failed to queue the browser WebTransport connection")
            get_tree().quit(2)
        return
    var options = ClassDB.instantiate("WebTransportTlsOptions")
    var custom_ca_path := OS.get_environment("GWT_TEST_CUSTOM_CA")
    if not custom_ca_path.is_empty():
        options.custom_ca_pem = FileAccess.get_file_as_bytes(custom_ca_path)
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

func _on_connection_succeeded(connected_session: RefCounted) -> void:
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
    print("godot-wtransport %s round trip passed" % ("Web" if OS.has_feature("web") else "native"))
    get_tree().quit(0)

func _parse_hash(value: String) -> PackedByteArray:
    var compact := value.replace(":", "")
    var result := PackedByteArray()
    if compact.length() != 64:
        return result
    for index in range(0, compact.length(), 2):
        result.append(compact.substr(index, 2).hex_to_int())
    return result

func _web_query_parameter(name: String, fallback: String) -> String:
    var window := JavaScriptBridge.get_interface("window")
    if not is_instance_valid(window):
        return fallback
    var search := str(window.location.search).trim_prefix("?")
    for pair in search.split("&", false):
        var parts := pair.split("=", true, 1)
        if parts.size() == 2 and parts[0].uri_decode() == name:
            return parts[1].replace("+", " ").uri_decode()
    return fallback
