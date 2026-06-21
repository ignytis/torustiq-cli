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

// TODO: implement stage creation and config value setting

TorustiqPluginStageHandle
TorustiqCli::Plugins::Builtin::File::CreateNewStage() {
    return 0;
}

void TorustiqCli::Plugins::Builtin::File::SetConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {

}

// no action needed on initialization
const TorustiqPlugin TorustiqCli::Plugins::Builtin::File::InitPlugin() {
    return TorustiqPlugin{
        .fn_create_new_stage = CreateNewStage,
        .fn_set_config_value = SetConfigValue,
    };
}
