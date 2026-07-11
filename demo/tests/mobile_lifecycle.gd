extends SceneTree

var reasons: Array[String] = []
var resumed := false

func _initialize() -> void:
    var client := WebTransportClient.new()
    root.add_child(client)
    client.sessions_closed_for_lifecycle.connect(_on_sessions_closed)
    client.application_resumed.connect(_on_resumed)
    assert(client.close_on_application_pause)
    client.notification(Node.NOTIFICATION_APPLICATION_PAUSED)
    client.notification(Node.NOTIFICATION_APPLICATION_RESUMED)
    assert(client.handle_network_change() == 0)
    await process_frame
    assert("application_paused" in reasons)
    assert("network_changed" in reasons)
    assert(resumed)
    print("godot-wtransport mobile lifecycle policy passed")
    quit(0)

func _on_sessions_closed(reason: String, count: int) -> void:
    assert(count == 0)
    reasons.append(reason)

func _on_resumed() -> void:
    resumed = true
