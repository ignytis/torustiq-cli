
#include "file.hpp"

#include <spdlog/spdlog.h>
#include <torustiq_sdk/plugins/typedefs.h>

#include <fstream>
#include <iostream>
#include <map>
#include <string>

#include "../../../defs.hpp"

using namespace std;

namespace {

/**
 * Represents an instance of a file stage.
 */
class StageInstance {
   public:
    StageInstance(bool writer = false);
    TorustiqPluginStageHandle stageHandle;
    string path;
    bool isWriter;  // true -> write file; false -> read file
};

StageInstance::StageInstance(bool writer) : isWriter(writer) {}

map<TorustiqPluginStageHandle, StageInstance> stageInstances;

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

void TorustiqCli::Plugins::Builtin::File::CreateNewStage(
    CreateNewStageFnArgs args) {
    // TODO: error handling. What if processor kind is passed? We have to return
    // an error.
    StageInstance newInstance;
    newInstance.isWriter = (args.stageKind == TORUSTIQ_PLUGIN_STAGE_KIND_SINK);
    newInstance.stageHandle = args.stageHandle;

    stageInstances[args.stageHandle] = newInstance;
}

void TorustiqCli::Plugins::Builtin::File::SetStageConfigValue(
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

void startReader(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    // Open file for reading
    spdlog::debug("file :: Starting reader stage with handle {}", stageHandle);
    ifstream file(instance->path);
    string line;
    while (getline(file, line)) {
        TorustiqMessage msg{};
        msg.type = TORUSTIQ_MESSAGE_TYPE_DATA;
        msg.payload_size = line.size();
        msg.payload = reinterpret_cast<uint8_t*>(line.data());
        hostGlobals.sendMessageFnPtr(stageHandle, &msg);
    }

    // Notify about end of file
    // TODO: bring to some function to avoid code duplication with other
    // plugins?
    TorustiqMessage msg{};
    msg.type = TORUSTIQ_MESSAGE_TYPE_EOF;
    hostGlobals.sendMessageFnPtr(stageHandle, &msg);

    spdlog::debug("file :: Reached EOF in reader with handle {}", stageHandle);
}

void startWriter(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    // Open file for writing
    spdlog::debug("file :: Starting writer stage with handle {}", stageHandle);
    ofstream file(instance->path);
}

}  // namespace

void TorustiqCli::Plugins::Builtin::File::Start(
    TorustiqPluginStageHandle stageHandle) {
    if (!stageInstances.contains(stageHandle)) {
        spdlog::error("stdio :: Stage handle not found: {}", stageHandle);
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
        .fn_stage_create_new = CreateNewStage,
        .fn_stage_set_config_value = SetStageConfigValue,
        .fn_stage_start = Start,
    };
}
