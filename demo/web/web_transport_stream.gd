class_name WebTransportWebStream
extends RefCounted

signal data_received(data: PackedByteArray)
signal finished
signal failed(error: Dictionary)

var handle := 0
var writable := false
var _client: WebTransportWebClient

func _configure(client: WebTransportWebClient, stream_handle: int, is_writable: bool) -> void:
	_client = client
	handle = stream_handle
	writable = is_writable

func write(data: PackedByteArray) -> Error:
	if not writable or not is_instance_valid(_client):
		return ERR_UNAVAILABLE
	_client._write_stream(handle, data)
	return OK

func finish() -> Error:
	if not writable or not is_instance_valid(_client):
		return ERR_UNAVAILABLE
	writable = false
	_client._finish_stream(handle)
	return OK
