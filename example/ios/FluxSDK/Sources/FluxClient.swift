import Foundation
import FluxFFI

public typealias SubscriptionID = UInt64

public final class FluxClient: @unchecked Sendable {

    private let handle: OpaquePointer

    public init() {
        guard let h = flux_create() else {
            fatalError("flux_create failed: \(FluxClient.lastError ?? "unknown")")
        }
        self.handle = h
    }

    deinit {
        flux_free(handle)
    }

    // MARK: - Server URL

    public var serverURL: String? {
        guard let cstr = flux_server_url(handle) else { return nil }
        return String(cString: cstr)
    }

    // MARK: - State

    public func get<T: Decodable>(_ path: String) -> T? {
        let bytes = flux_get(handle, path)
        defer { flux_bytes_free(bytes) }

        guard let ptr = bytes.ptr, bytes.len > 0 else { return nil }

        let data = Data(bytes: ptr, count: bytes.len)
        return try? JSONDecoder().decode(T.self, from: data)
    }

    public func getRawJSON(_ path: String) -> Data? {
        let bytes = flux_get(handle, path)
        defer { flux_bytes_free(bytes) }

        guard let ptr = bytes.ptr, bytes.len > 0 else { return nil }
        return Data(bytes: ptr, count: bytes.len)
    }

    // MARK: - Requests

    public func emit(_ path: String, payload: some Encodable) {
        guard let data = try? JSONEncoder().encode(payload),
              let json = String(data: data, encoding: .utf8)
        else { return }
        flux_emit(handle, path, json)
    }

    public func emit(_ path: String) {
        flux_emit(handle, path, "{}")
    }

    // MARK: - Subscriptions

    private static var callbacks: [SubscriptionID: (String, Data?) -> Void] = [:]
    private static let callbackLock = NSLock()

    public func subscribe(
        _ pattern: String,
        onChange: @escaping (_ path: String, _ json: Data?) -> Void
    ) -> SubscriptionID {
        let wrapper: @convention(c) (
            UnsafePointer<CChar>?,
            UnsafePointer<CChar>?
        ) -> Void = { pathPtr, jsonPtr in
            guard let pathPtr else { return }
            let path = String(cString: pathPtr)
            var jsonData: Data?
            if let jsonPtr {
                let jsonStr = String(cString: jsonPtr)
                jsonData = jsonStr.data(using: .utf8)
            }
            FluxClient.callbackLock.lock()
            let allCallbacks = FluxClient.callbacks
            FluxClient.callbackLock.unlock()
            for (_, cb) in allCallbacks {
                cb(path, jsonData)
            }
        }

        let subID = flux_subscribe(handle, pattern, wrapper)
        if subID != 0 {
            FluxClient.callbackLock.lock()
            FluxClient.callbacks[subID] = onChange
            FluxClient.callbackLock.unlock()
        }
        return subID
    }

    public func unsubscribe(_ id: SubscriptionID) {
        flux_unsubscribe(handle, id)
        FluxClient.callbackLock.lock()
        FluxClient.callbacks.removeValue(forKey: id)
        FluxClient.callbackLock.unlock()
    }

    // MARK: - i18n

    public func i18n(_ key: String) -> String? {
        let bytes = flux_i18n_get(handle, key)
        defer { flux_bytes_free(bytes) }

        guard let ptr = bytes.ptr, bytes.len > 0 else { return nil }
        return String(bytes: Data(bytes: ptr, count: bytes.len), encoding: .utf8)
    }

    public func setLocale(_ locale: String) {
        flux_i18n_set_locale(handle, locale)
    }

    // MARK: - Error

    public static var lastError: String? {
        guard let cstr = flux_last_error() else { return nil }
        return String(cString: cstr)
    }
}
