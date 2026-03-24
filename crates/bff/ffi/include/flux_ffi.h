#ifndef FLUX_FFI_H
#define FLUX_FFI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- Opaque types ---- */

typedef struct FluxHandle FluxHandle;

/* ---- Data types ---- */

typedef struct {
    const uint8_t *ptr;
    size_t         len;
} FluxBytes;

/* Callback for state change notifications.
 * `path` — the state path that changed (null-terminated).
 * `json` — the new value serialized as JSON (null-terminated). */
typedef void (*FluxChangeCallback)(const char *path, const char *json);

/* ---- Lifecycle ---- */

FluxHandle *flux_create(void);

void flux_free(FluxHandle *handle);

const char *flux_server_url(const FluxHandle *handle);

/* ---- State ---- */

FluxBytes flux_get(const FluxHandle *handle, const char *path);

void flux_bytes_free(FluxBytes bytes);

/* ---- Requests ---- */

void flux_emit(FluxHandle *handle, const char *path, const char *payload_json);

/* ---- I18n ---- */

FluxBytes flux_i18n_get(const FluxHandle *handle, const char *url);

void flux_i18n_set_locale(const FluxHandle *handle, const char *locale);

/* ---- Subscriptions ---- */

uint64_t flux_subscribe(FluxHandle *handle,
                        const char *pattern,
                        FluxChangeCallback callback);

void flux_unsubscribe(FluxHandle *handle, uint64_t subscription_id);

/* ---- Error ---- */

const char *flux_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* FLUX_FFI_H */
