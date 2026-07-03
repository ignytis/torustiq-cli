#include "stdio.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

#include <cstring>
#include <iostream>
#include <vector>

#include "../../../defs.hpp"

using namespace std;

class StageInstance {
   public:
    bool isWriter;  // true -> stdout, false -> stdin
};

namespace {
vector<StageInstance> stageInstances;
TorustiqHostGlobals hostGlobals;
}  // namespace

const TorustiqPluginInfo TorustiqCli::Plugins::Builtin::Stdio::GetPluginInfo() {
    return TorustiqPluginInfo{
        .host_app = APP_NAME,
        .api_version = API_VERSION,
        .id = "stdio",
        .name = "Standard IO Plugin",
    };
}

// TODO: implement stage creation and config value setting

TorustiqPluginStageHandle TorustiqCli::Plugins::Builtin::Stdio::CreateNewStage(
    CreateNewStageFnArgs args) {
    return 0;
}

void TorustiqCli::Plugins::Builtin::Stdio::SetConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {

}

namespace {

void startReader(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    string line;
    while (std::getline(cin, line)) {
        TorustiqMessage* msg =
            (TorustiqMessage*)malloc(sizeof(TorustiqMessage));
        msg->payload_size = line.size();
        msg->payload = (uint8_t*)malloc(msg->payload_size);
        memcpy(msg->payload, line.c_str(), msg->payload_size);
        hostGlobals.sendMessageFnPtr(stageHandle, msg);
        free(msg);
    }
}
}  // namespace

void startWriter(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {}

void TorustiqCli::Plugins::Builtin::Stdio::Start(
    TorustiqPluginStageHandle stageHandle) {
    if (stageHandle >= stageInstances.size()) {
        return;
    }
    StageInstance& instance = stageInstances[stageHandle];

    if (instance.isWriter) {
        startWriter(stageHandle, &instance);
    } else {
        startReader(stageHandle, &instance);
    }
}

const TorustiqPlugin TorustiqCli::Plugins::Builtin::Stdio::InitPlugin(
    TorustiqHostGlobals globals) {
    hostGlobals = globals;
    return TorustiqPlugin{
        .fn_create_new_stage = CreateNewStage,
        .fn_set_config_value = SetConfigValue,
        .fn_stage_start = Start,
    };
}
