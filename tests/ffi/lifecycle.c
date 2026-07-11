#include "godot_wtransport.h"

#include <assert.h>
#include <stddef.h>

int main(void) {
    assert(gwt_abi_version() == GWT_ABI_VERSION);
    for (size_t index = 0; index < 100; ++index) {
        GwtClient *client = gwt_client_create(16);
        assert(client != NULL);
        GwtClientStats stats = {0};
        assert(gwt_client_stats(client, &stats) == GWT_STATUS_OK);
        assert(stats.active_sessions == 0);
        gwt_client_destroy(client);
    }
    return 0;
}
