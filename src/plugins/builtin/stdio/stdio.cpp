#include "stdio.hpp"

#include "../../../defs.hpp"

const TorustiqPluginInfo TorustiqCli::Plugins::Builtin::Stdio::GetPluginInfo() {
    return TorustiqPluginInfo{
        .host_app = APP_NAME,
        .api_version = API_VERSION,
        .id = "stdio",
        .name = "Standard IO Plugin",
    };
}

// no action needed on initialization
void TorustiqCli::Plugins::Builtin::Stdio::InitPlugin() {}
