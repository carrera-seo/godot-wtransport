class_name WebTransportWebClient
extends Node

signal connection_succeeded(session: WebTransportWebSession)
signal connection_failed(error: Dictionary)

var _bridge: JavaScriptObject
var _sessions: Dictionary = {}
var _streams: Dictionary = {}

static func is_supported() -> bool:
	if not OS.has_feature("web"):
		return false
	return JavaScriptBridge.eval("typeof WebTransport === 'function' && globalThis.isSecureContext === true")

func _ready() -> void:
	if not OS.has_feature("web"):
		set_process(false)
		return
	var source := FileAccess.get_file_as_string("res://web/godot_wtransport_web.js")
	JavaScriptBridge.eval(source, true)
	_bridge = JavaScriptBridge.get_interface("GodotWebTransportBridge")
	set_process(is_instance_valid(_bridge))

func connect_to_url(url: String, certificate_hashes: Array[PackedByteArray] = []) -> int:
	if not is_instance_valid(_bridge) or not is_supported():
		connection_failed.emit(_error("unsupported", "WebTransport requires a supported browser and secure context."))
		return 0
	var hashes: Array[String] = []
	for hash_bytes in certificate_hashes:
		if hash_bytes.size() != 32:
			connection_failed.emit(_error("certificate_hash", "Each SHA-256 certificate hash must contain 32 bytes."))
			return 0
		hashes.append(Marshalls.raw_to_base64(hash_bytes))
	_bridge.beginConnection(url, JSON.stringify(hashes))
	return 1

func _process(_delta: float) -> void:
	var parsed = JSON.parse_string(str(_bridge.poll()))
	if parsed is not Array:
		return
	for event in parsed:
		_dispatch(event)

func _dispatch(event: Dictionary) -> void:
	var session_id := int(event.get("session", 0))
	match event.get("type", ""):
		"connected":
			var session := WebTransportWebSession.new()
			session._configure(self, session_id)
			session.reliability = event.get("reliability", "pending")
			_sessions[session_id] = session
			connection_succeeded.emit(session)
		"datagram":
			_session(session_id).datagram_received.emit(Marshalls.base64_to_raw(event.data))
		"incoming_bidi", "incoming_uni", "stream_opened":
			_dispatch_stream(event)
		"stream_data":
			_stream(int(event.stream)).data_received.emit(Marshalls.base64_to_raw(event.data))
		"stream_finished":
			_stream(int(event.stream)).finished.emit()
		"draining":
			_session(session_id).draining_started.emit()
		"closed":
			_session(session_id).closed.emit({"code": int(event.get("code", 0)), "reason": event.get("reason", "")})
		"error":
			var error := _error(event.get("operation", "browser"), event.get("message", "Unknown browser error"), event.get("name", "Error"))
			if event.get("stream", 0) != 0:
				_stream(int(event.stream)).failed.emit(error)
			elif _sessions.has(session_id):
				_session(session_id).failed.emit(error)
			else:
				connection_failed.emit(error)

func _dispatch_stream(event: Dictionary) -> void:
	var stream_id := int(event.stream)
	var stream := WebTransportWebStream.new()
	var writable: bool = event.get("type", "") != "incoming_uni"
	stream._configure(self, stream_id, writable)
	_streams[stream_id] = stream
	var session := _session(int(event.session))
	match event.type:
		"incoming_bidi": session.incoming_bidirectional_stream.emit(stream)
		"incoming_uni": session.incoming_unidirectional_stream.emit(stream)
		"stream_opened": session.stream_opened.emit(stream)

func _session(handle: int) -> WebTransportWebSession:
	return _sessions.get(handle) as WebTransportWebSession

func _stream(handle: int) -> WebTransportWebStream:
	return _streams.get(handle) as WebTransportWebStream

func _send_datagram(session: int, data: PackedByteArray) -> void:
	_bridge.sendDatagram(session, Marshalls.raw_to_base64(data))

func _open_stream(session: int, bidirectional: bool) -> void:
	_bridge.openStream(session, bidirectional)

func _write_stream(stream: int, data: PackedByteArray) -> void:
	_bridge.writeStream(stream, Marshalls.raw_to_base64(data))

func _finish_stream(stream: int) -> void:
	_bridge.finishStream(stream)

func _close_session(session: int, code: int, reason: String) -> void:
	_bridge.close(session, code, reason)

func _error(operation: String, message: String, name := "Error") -> Dictionary:
	return {"category": "browser", "operation": operation, "name": name, "message": message, "retryable": false}
