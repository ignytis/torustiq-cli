#include "stdio.hpp"

#include <spdlog/spdlog.h>
#include <torustiq_sdk/message.h>
#include <torustiq_sdk/typedefs.h>

#include <algorithm>
#include <cstring>
#include <iostream>
#include <map>

#include "../../../common/strings.hpp"
#include "../../../defs.hpp"

using namespace std;
using namespace TorustiqCli::Common;

namespace {

/**
 * Represents an instance of a standard input/output stage.
 */
class StageInstance {
   public:
    StageInstance(TorustiqPluginStageHandle stageHandle = 0,
                  bool writer = false);
    TorustiqPluginStageHandle stageHandle;
    bool isWriter;  // true -> stdout, false -> stdin
    /**
     * Output format of messages. It has placeholders, like:
     * %P - payload
     */
    string outputFormat;
};

StageInstance::StageInstance(TorustiqPluginStageHandle stageHandle, bool writer)
    : stageHandle(stageHandle), isWriter(writer), outputFormat("%P") {}

map<TorustiqPluginStageHandle, StageInstance> stageInstances;
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

void TorustiqCli::Plugins::Builtin::Stdio::CreateNewStage(
    CreateNewStageFnArgs args) {
    // TODO: error handling. What if processor kind is passed? We have to return
    // an error.

    StageInstance newInstance(
        args.stageHandle, args.stageKind == TORUSTIQ_PLUGIN_STAGE_KIND_SINK);
    stageInstances[args.stageHandle] = newInstance;
}

void TorustiqCli::Plugins::Builtin::Stdio::SetStageConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {
    if (stageHandle >= stageInstances.size()) {
        return;
    }
    StageInstance& instance = stageInstances[stageHandle];
    if (string(key) == "output_format") {
        instance.outputFormat = string(value);
    }
}

namespace {

void startReader(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    spdlog::debug("stdio :: Starting reader stage with handle {}", stageHandle);

    string line;
    while (std::getline(cin, line)) {
        TorustiqMessage msg{};
        msg.type = TORUSTIQ_MESSAGE_TYPE_DATA;
        msg.payload_size = line.size();
        msg.payload = (uint8_t*)malloc(msg.payload_size);
        if (msg.payload != nullptr) {
            memcpy(msg.payload, line.c_str(), msg.payload_size);
        }
        msg.headers_count = 0;
        msg.headers = nullptr;
        hostGlobals.sendMessageFnPtr(stageHandle, &msg);
        free(msg.payload);
    }

    TorustiqMessage msg = torustiq_message_create_eof();
    hostGlobals.sendMessageFnPtr(stageHandle, &msg);
}
}  // namespace

void startWriter(TorustiqPluginStageHandle stageHandle,
                 StageInstance* instance) {
    spdlog::debug("stdio :: Starting writer stage with handle {}", stageHandle);

    while (true) {
        // TODO: free memory? Perhaps will need to deep-copy the message +
        // call a function from host
        const TorustiqMessage* msg =
            hostGlobals.receiveMessageFnPtr(stageHandle);
        if (msg == nullptr) {
            spdlog::error(
                "stdio :: Empty pointer received");  // todo: figure out mode
                                                     // reasonable message
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_EOF) {
            spdlog::debug(
                "stdio :: Received EOF message. Exiting writer stage.");
            torustiq_message_free(const_cast<TorustiqMessage*>(msg));
            break;
        }

        if (msg->type == TORUSTIQ_MESSAGE_TYPE_DATA) {
            char* payloadStr = new char[msg->payload_size + 1];
            memcpy(payloadStr, msg->payload, msg->payload_size);
            payloadStr[msg->payload_size] = '\0';
            string payload(payloadStr);
            string output(instance->outputFormat);
            output = Strings::replaceAll(output, "%P", payload);
            cout << output << endl;
            delete payloadStr;
        }

        torustiq_message_free(const_cast<TorustiqMessage*>(msg));
    }
}

void TorustiqCli::Plugins::Builtin::Stdio::Start(
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
