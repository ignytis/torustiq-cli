
#include "file.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

#include <cstring>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>

#include "../../../defs.hpp"

using namespace std;

class StageInstance {
   public:
    StageInstance(bool writer = false);
    string path;
    bool isWriter;  // true -> write file; false -> read file
};

StageInstance::StageInstance(bool writer) : isWriter(writer) {}

namespace {
vector<StageInstance> stageInstances;
TorustiqHostGlobals hostGlobals;
}  // namespace

const TorustiqPluginInfo TorustiqCli::Plugins::Builtin::File::GetPluginInfo() {
    return TorustiqPluginInfo{
        .host_app = APP_NAME,
        .api_version = API_VERSION,
        .id = "file",
        .name = "File IO Plugin",
    };
}

TorustiqPluginStageHandle TorustiqCli::Plugins::Builtin::File::CreateNewStage(
    CreateNewStageFnArgs args) {
    // TODO: error handling. What if processor kind is passed? We have to return
    // an error.
    StageInstance newInstance;
    newInstance.isWriter = (args.stageKind == TORUSTIQ_PLUGIN_STAGE_KIND_SINK);
    stageInstances.push_back(newInstance);
    return stageInstances.size() - 1;
}

void TorustiqCli::Plugins::Builtin::File::SetConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {
    if (stageHandle >= stageInstances.size()) {
        return;
    }
    StageInstance& instance = stageInstances[stageHandle];
    if (string(key) == "path") {
        instance.path = string(value);
    }
}

namespace {

// TODO: delete
void startReader(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    // Open file for reading
    ifstream file(instance->path);
    string line;
    while (getline(file, line)) {
        // TODO:
        // 1. create a new instance of message
        // 2. deallocate memory
        TorustiqMessage* msg =
            (TorustiqMessage*)malloc(sizeof(TorustiqMessage));
        msg->payload_size = line.size();
        msg->payload = (uint8_t*)malloc(msg->payload_size);
        memcpy(msg->payload, line.c_str(), msg->payload_size);
        hostGlobals.sendMessageFnPtr(stageHandle, msg);
        free(msg);
        cout << "Read line: " << line << endl;
    }
}

void startWriter(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    // Open file for writing
    ofstream file(instance->path);
}

}  // namespace

void TorustiqCli::Plugins::Builtin::File::Start(
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

// no action needed on initialization
const TorustiqPlugin TorustiqCli::Plugins::Builtin::File::InitPlugin(
    TorustiqHostGlobals globals) {
    hostGlobals = globals;
    return TorustiqPlugin{
        .fn_create_new_stage = CreateNewStage,
        .fn_set_config_value = SetConfigValue,
        .fn_stage_start = Start,
    };
}
