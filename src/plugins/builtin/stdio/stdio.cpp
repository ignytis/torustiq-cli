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

// TODO: implement stage creation and config value setting

TorustiqPluginStageHandle
TorustiqCli::Plugins::Builtin::Stdio::CreateNewStage() {
    return 0;
}

void TorustiqCli::Plugins::Builtin::Stdio::SetConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {

}

// no action needed on initialization
const TorustiqPlugin TorustiqCli::Plugins::Builtin::Stdio::InitPlugin() {
    return TorustiqPlugin{
        .fn_create_new_stage = CreateNewStage,
        .fn_set_config_value = SetConfigValue,
    };
}
