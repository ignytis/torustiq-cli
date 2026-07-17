
#include "file.hpp"

#include <spdlog/spdlog.h>
#include <torustiq_sdk/message.h>
#include <torustiq_sdk/typedefs.h>

#include <fstream>
#include <iostream>
#include <map>
#include <string>

#include "../../../defs.hpp"
#include "file_reader.hpp"
#include "typedefs.hpp"

using namespace std;

using TorustiqCli::Plugins::Builtin::File::StageInstance;
using TorustiqCli::Plugins::Builtin::File::StageInstanceMode;
using TorustiqCli::Plugins::Builtin::File::startReader;

namespace {

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
    StageInstanceMode mode = args.stageKind == TORUSTIQ_PLUGIN_STAGE_KIND_SINK
                                 ? StageInstanceMode::WRITER
                                 : StageInstanceMode::READER;
    StageInstance newInstance(args.stageHandle, mode, hostGlobals);
    stageInstances.insert({args.stageHandle, newInstance});
}

void TorustiqCli::Plugins::Builtin::File::SetStageConfigValue(
    TorustiqPluginStageHandle stageHandle, const char* key, const char* value) {
    if (stageHandle >= stageInstances.size()) {
        return;
    }
    StageInstance& instance = stageInstances.at(stageHandle);
    string strKey = string(key);
    string strValue = string(value);
    if (strKey == "path") {
        instance.path = strValue;
    } else if (strKey == "contents_mode") {
        if (strValue == "file_lines") {
            instance.contentsMode = StageInstanceContentsMode::FILE_LINES;
        } else if (strValue == "directory_files") {
            instance.contentsMode = StageInstanceContentsMode::DIRECTORY_FILES;
        } else {
            spdlog::error(
                "file :: Unrecognized contents mode '{}' for stage with handle "
                "{}",
                strValue, stageHandle);
            exit(EXIT_FAILURE);
        }
    }
}

namespace {

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

    StageInstance& instance = stageInstances.at(stageHandle);
    switch (instance.mode) {
        case READER:
            startReader(&instance);
            break;
        case WRITER:
            startWriter(stageHandle, &instance);
            break;
        default:
            spdlog::error("file :: unreachable code");
            exit(EXIT_FAILURE);
            break;
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
