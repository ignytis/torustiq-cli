#include "file.hpp"

#include "../../../defs.hpp"

const TorustiqPluginInfo TorustiqCli::Plugins::Builtin::File::GetPluginInfo() {
    return TorustiqPluginInfo{
        .host_app = APP_NAME,
        .api_version = API_VERSION,
        .id = "file",
        .name = "File IO Plugin",
    };
}

// no action needed on initialization
void TorustiqCli::Plugins::Builtin::File::InitPlugin() {}
