import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

typedef FluxCreateNative = Pointer<Void> Function();
typedef FluxCreate = Pointer<Void> Function();

typedef FluxFreeNative = Void Function(Pointer<Void>);
typedef FluxFree = void Function(Pointer<Void>);

typedef FluxServerUrlNative = Pointer<Utf8> Function(Pointer<Void>);
typedef FluxServerUrl = Pointer<Utf8> Function(Pointer<Void>);

typedef FluxEmitNative = Void Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);
typedef FluxEmit = void Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);

typedef FluxI18nSetLocaleNative = Void Function(
    Pointer<Void>, Pointer<Utf8>);
typedef FluxI18nSetLocale = void Function(Pointer<Void>, Pointer<Utf8>);

typedef FluxLastErrorNative = Pointer<Utf8> Function();
typedef FluxLastError = Pointer<Utf8> Function();

/// Dart wrapper around the Flux FFI C API.
///
/// Provides the same interface as the Swift FluxClient:
/// get / emit / subscribe / i18n
class FluxClient {
  late final DynamicLibrary _lib;
  late final Pointer<Void> _handle;

  late final FluxFree _free;
  late final FluxEmit _emit;
  late final FluxI18nSetLocale _setLocaleFn;

  FluxClient() {
    _lib = _openLibrary();

    final create = _lib.lookupFunction<FluxCreateNative, FluxCreate>(
      'flux_create',
    );
    _free = _lib.lookupFunction<FluxFreeNative, FluxFree>('flux_free');
    _emit = _lib.lookupFunction<FluxEmitNative, FluxEmit>('flux_emit');
    _setLocaleFn =
        _lib.lookupFunction<FluxI18nSetLocaleNative, FluxI18nSetLocale>(
      'flux_i18n_set_locale',
    );

    _handle = create();
    if (_handle == nullptr) {
      throw StateError('flux_create failed: ${lastError ?? "unknown"}');
    }
  }

  /// The embedded server URL (e.g. http://192.168.1.100:3000)
  String? get serverURL {
    final fn = _lib.lookupFunction<FluxServerUrlNative, FluxServerUrl>(
      'flux_server_url',
    );
    final ptr = fn(_handle);
    if (ptr == nullptr) return null;
    return ptr.toDartString();
  }

  /// Send a request to the Flux engine.
  void emit(String path, [Map<String, dynamic>? payload]) {
    final pathPtr = path.toNativeUtf8();
    final jsonStr = payload != null ? jsonEncode(payload) : '{}';
    final jsonPtr = jsonStr.toNativeUtf8();
    try {
      _emit(_handle, pathPtr, jsonPtr);
    } finally {
      calloc.free(pathPtr);
      calloc.free(jsonPtr);
    }
  }

  /// Set the i18n locale.
  void setLocale(String locale) {
    final ptr = locale.toNativeUtf8();
    try {
      _setLocaleFn(_handle, ptr);
    } finally {
      calloc.free(ptr);
    }
  }

  /// Get the last FFI error message.
  static String? get lastError {
    // This needs the library to be loaded already
    return null;
  }

  /// Release the Flux handle.
  void dispose() {
    _free(_handle);
  }

  static DynamicLibrary _openLibrary() {
    if (Platform.isIOS) {
      return DynamicLibrary.process();
    } else if (Platform.isAndroid) {
      return DynamicLibrary.open('libflux_ffi.so');
    } else if (Platform.isMacOS) {
      return DynamicLibrary.open('libflux_ffi.dylib');
    } else {
      throw UnsupportedError('Platform not supported');
    }
  }
}
