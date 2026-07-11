extends SceneTree

var client: WebTransportClient
var options: WebTransportTlsOptions
var stage := 0
var pause_signal := false
var network_signal := false
var resume_signal := false

func _initialize() -> void:
    client = WebTransportClient.new()
    root.add_child(client)
    client.connection_succeeded.connect(_on_connected)
    client.connection_failed.connect(_on_connection_failed)
    client.sessions_closed_for_lifecycle.connect(_on_sessions_closed)
    client.application_resumed.connect(func() -> void: resume_signal = true)
    options = WebTransportTlsOptions.new()
    options.custom_ca_pem = FileAccess.get_file_as_bytes(OS.get_environment("GWT_TEST_CUSTOM_CA"))
    _connect()
    create_timer(15.0).timeout.connect(_on_timeout)

func _connect() -> void:
    assert(client.connect_to_url(OS.get_environment("GWT_TEST_URL"), options) != 0)

func _on_connected(session: WebTransportSession) -> void:
    session.closed.connect(_on_closed)
    if stage == 0:
        client.notification(Node.NOTIFICATION_APPLICATION_PAUSED)
        client.notification(Node.NOTIFICATION_APPLICATION_RESUMED)
    else:
        assert(client.handle_network_change() == 1)

func _on_closed(_close_info: Dictionary) -> void:
    if stage == 0:
        assert(pause_signal)
        assert(resume_signal)
        stage = 1
        _connect()
    else:
        assert(network_signal)
        print("godot-wtransport active mobile lifecycle passed")
        quit(0)

func _on_sessions_closed(reason: String, count: int) -> void:
    assert(count == 1)
    if reason == "application_paused":
        pause_signal = true
    elif reason == "network_changed":
        network_signal = true

func _on_connection_failed(error: Dictionary) -> void:
    push_error("Mobile lifecycle connection failed: %s" % error)
    quit(2)

func _on_timeout() -> void:
    push_error("Active mobile lifecycle test timed out")
    quit(3)
