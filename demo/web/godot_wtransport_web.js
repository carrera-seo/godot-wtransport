(function () {
  "use strict";

  if (globalThis.GodotWebTransportBridge) return;

  const sessions = new Map();
  const streams = new Map();
  const events = [];
  let nextSession = 1;
  let nextStream = 1;

  const bytesToBase64 = (bytes) => {
    let binary = "";
    for (let offset = 0; offset < bytes.length; offset += 0x8000) {
      binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000));
    }
    return btoa(binary);
  };
  const base64ToBytes = (value) => Uint8Array.from(atob(value), (char) => char.charCodeAt(0));
  const emit = (event) => events.push(event);
  const errorInfo = (error, operation, session = 0, stream = 0) => ({
    type: "error", operation, session, stream,
    name: error && error.name ? String(error.name) : "Error",
    message: error && error.message ? String(error.message) : String(error),
  });

  async function readStream(session, stream, readable) {
    const reader = readable.getReader();
    try {
      while (true) {
        const result = await reader.read();
        if (result.done) break;
        emit({ type: "stream_data", session, stream, data: bytesToBase64(result.value) });
      }
      emit({ type: "stream_finished", session, stream });
    } catch (error) {
      emit(errorInfo(error, "read_stream", session, stream));
    } finally {
      reader.releaseLock();
    }
  }

  async function acceptStreams(session, readable, bidirectional) {
    const reader = readable.getReader();
    try {
      while (true) {
        const result = await reader.read();
        if (result.done) break;
        const stream = nextStream++;
        const value = result.value;
        const source = bidirectional ? value.readable : value;
        const writer = bidirectional ? value.writable.getWriter() : null;
        streams.set(stream, { session, writer });
        emit({ type: bidirectional ? "incoming_bidi" : "incoming_uni", session, stream });
        void readStream(session, stream, source);
      }
    } catch (error) {
      emit(errorInfo(error, bidirectional ? "accept_bidi" : "accept_uni", session));
    } finally {
      reader.releaseLock();
    }
  }

  async function readDatagrams(session, transport) {
    const reader = transport.datagrams.readable.getReader();
    try {
      while (true) {
        const result = await reader.read();
        if (result.done) break;
        emit({ type: "datagram", session, data: bytesToBase64(result.value) });
      }
    } catch (error) {
      emit(errorInfo(error, "read_datagram", session));
    } finally {
      reader.releaseLock();
    }
  }

  const api = {
    supported: () => typeof globalThis.WebTransport === "function" && globalThis.isSecureContext === true,
    beginConnection: async (url, hashesJson) => {
      const session = nextSession++;
      try {
        const hashes = JSON.parse(hashesJson || "[]").map((value) => ({
          algorithm: "sha-256", value: base64ToBytes(value),
        }));
        const options = hashes.length ? { serverCertificateHashes: hashes } : {};
        const transport = new WebTransport(url, options);
        sessions.set(session, { transport, datagramWriter: transport.datagrams.writable.getWriter() });
        await transport.ready;
        emit({ type: "connected", session, reliability: String(transport.reliability || "pending") });
        void readDatagrams(session, transport);
        void acceptStreams(session, transport.incomingBidirectionalStreams, true);
        void acceptStreams(session, transport.incomingUnidirectionalStreams, false);
        transport.draining.then(() => emit({ type: "draining", session })).catch(() => {});
        transport.closed.then((info) => {
          emit({ type: "closed", session, code: info.closeCode || 0, reason: info.reason || "" });
          sessions.delete(session);
        }).catch((error) => emit(errorInfo(error, "closed", session)));
      } catch (error) {
        sessions.delete(session);
        emit(errorInfo(error, "connect", session));
      }
      return session;
    },
    sendDatagram: async (session, data) => {
      try { await sessions.get(session).datagramWriter.write(base64ToBytes(data)); }
      catch (error) { emit(errorInfo(error, "send_datagram", session)); }
    },
    openStream: async (session, bidirectional) => {
      try {
        const transport = sessions.get(session).transport;
        const value = bidirectional ? await transport.createBidirectionalStream() : await transport.createUnidirectionalStream();
        const stream = nextStream++;
        const writer = (bidirectional ? value.writable : value).getWriter();
        streams.set(stream, { session, writer });
        emit({ type: "stream_opened", session, stream, bidirectional });
        if (bidirectional) void readStream(session, stream, value.readable);
      } catch (error) { emit(errorInfo(error, "open_stream", session)); }
    },
    writeStream: async (stream, data) => {
      try { await streams.get(stream).writer.write(base64ToBytes(data)); }
      catch (error) { emit(errorInfo(error, "write_stream", 0, stream)); }
    },
    finishStream: async (stream) => {
      try { await streams.get(stream).writer.close(); }
      catch (error) { emit(errorInfo(error, "finish_stream", 0, stream)); }
    },
    close: (session, code, reason) => {
      const entry = sessions.get(session);
      if (entry) entry.transport.close({ closeCode: code, reason });
    },
    poll: () => JSON.stringify(events.splice(0, events.length)),
  };

  globalThis.GodotWebTransportBridge = api;
})();
