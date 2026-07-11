class_name WebTransportWebSession
extends RefCounted

signal draining_started
signal closed(close_info: Dictionary)
signal datagram_received(data: PackedByteArray)
signal incoming_bidirectional_stream(stream: WebTransportWebStream)
signal incoming_unidirectional_stream(stream: WebTransportWebStream)
signal stream_opened(stream: WebTransportWebStream)
signal failed(error: Dictionary)

var handle := 0
var reliability := "pending"
var _client: WebTransportWebClient

func _configure(client: WebTransportWebClient, session_handle: int) -> void:
	_client = client
	handle = session_handle

func send_datagram(data: PackedByteArray) -> Error:
	if not is_instance_valid(_client):
		return ERR_UNCONFIGURED
	_client._send_datagram(handle, data)
	return OK

func create_bidirectional_stream() -> Error:
	return _open_stream(true)

func create_unidirectional_stream() -> Error:
	return _open_stream(false)

func close(code := 0, reason := "") -> Error:
	if not is_instance_valid(_client):
		return ERR_UNCONFIGURED
	_client._close_session(handle, code, reason)
	return OK

func _open_stream(bidirectional: bool) -> Error:
	if not is_instance_valid(_client):
		return ERR_UNCONFIGURED
	_client._open_stream(handle, bidirectional)
	return OK
