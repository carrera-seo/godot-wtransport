extends SceneTree

func _initialize() -> void:
    call_deferred("_run")

func _run() -> void:
    for index in range(50):
        var client := WebTransportClient.new()
        root.add_child(client)
        client.queue_free()
        await process_frame
    print("godot-wtransport lifecycle smoke passed")
    quit(0)
