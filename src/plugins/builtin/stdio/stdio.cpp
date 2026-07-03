#include "stdio.hpp"

#include <spdlog/spdlog.h>
#include <torustiq_sdk/plugins/typedefs.h>

#include <cstring>
#include <iostream>
#include <vector>

#include "../../../defs.hpp"

using namespace std;

namespace {

/**
 * Represents an instance of a standard input/output stage.
 */
class StageInstance {
   public:
    bool isWriter;  // true -> stdout, false -> stdin
};

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

const TorustiqPlugin TorustiqCli::Plugins::Builtin::Stdio::InitPlugin(
    TorustiqHostGlobals globals) {
    hostGlobals = globals;
    return TorustiqPlugin{
        .fn_stage_create_new = CreateNewStage,
        .fn_stage_set_config_value = SetStageConfigValue,
        .fn_stage_start = Start,
    };
}

TorustiqPluginStageHandle TorustiqCli::Plugins::Builtin::Stdio::CreateNewStage(
    CreateNewStageFnArgs args) {
    // TODO: error handling. What if processor kind is passed? We have to return
    // an error.
    StageInstance newInstance;
    newInstance.isWriter = (args.stageKind == TORUSTIQ_PLUGIN_STAGE_KIND_SINK);
    stageInstances.push_back(newInstance);
    return stageInstances.size() - 1;
}

void TorustiqCli::Plugins::Builtin::Stdio::SetStageConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {
    if (stageHandle >= stageInstances.size()) {
        return;
    }
    StageInstance& instance = stageInstances[stageHandle];
    // TODO: add config here. At least we might have message format for output
    // here An idea: I/O mode: reading line by line vs processing the whole
    // stream
}

namespace {

void startReader(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    spdlog::debug("stdio :: Starting reader stage with handle {}", stageHandle);

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
                 StageInstance* instance) {
    spdlog::debug("stdio :: Starting writer stage with handle {}", stageHandle);

    while (true) {
        const TorustiqMessage* msg = hostGlobals.receiveMessageFnPtr(stageHandle);
        if (msg == nullptr) {
            spdlog::error("stdio :: Empty pointer received"); // todo: figure out mode reasonable message
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_EOF) {
            spdlog::debug("stdio :: Received EOF message. Exiting writer stage.");
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_DATA) {
            string line(reinterpret_cast<const char*>(msg->payload),
                        msg->payload_size);
            cout << line << endl;
        }
    }



    
}

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
