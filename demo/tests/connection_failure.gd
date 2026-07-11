extends SceneTree

func _initialize() -> void:
    var client := WebTransportClient.new()
    root.add_child(client)
    client.connection_failed.connect(_on_connection_failed)
    assert(client.connect_to_url("https://127.0.0.1:1") != 0)
    create_timer(15.0).timeout.connect(_on_timeout)

func _on_connection_failed(error: Dictionary) -> void:
    for key in ["category", "code", "message", "retryable", "transport_error_code", "http3_error_code", "stream_error_code", "tls_alert", "os_error"]:
        assert(error.has(key))
    print("godot-wtransport structured connection failure passed")
    quit(0)

func _on_timeout() -> void:
    push_error("Connection failure event timed out")
    quit(2)
