extends SceneTree

func _initialize() -> void:
    var client := WebTransportClient.new()
    root.add_child(client)
    client.connection_failed.connect(_on_connection_failed)
    var options := WebTransportTlsOptions.new()
    var custom_ca_path := OS.get_environment("GWT_TEST_CUSTOM_CA")
    if not custom_ca_path.is_empty():
        options.custom_ca_pem = FileAccess.get_file_as_bytes(custom_ca_path)
    assert(client.connect_to_url(OS.get_environment("GWT_TEST_URL"), options) != 0)
    create_timer(15.0).timeout.connect(_on_timeout)

func _on_connection_failed(error: Dictionary) -> void:
    assert(error.category == 3, "Expected TLS error category, received: %s" % error)
    var expected := OS.get_environment("GWT_TEST_ERROR_CONTAINS").to_lower()
    assert(expected.is_empty() or expected in String(error.message).to_lower(), "Unexpected TLS error: %s" % error)
    print("godot-wtransport TLS failure classification passed: %s" % error.message)
    quit(0)

func _on_timeout() -> void:
    push_error("TLS failure classification timed out")
    quit(2)
