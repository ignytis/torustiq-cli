#include "abstract_plugin.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

using namespace TorustiqCli::Plugins;

AbstractPlugin::AbstractPlugin(AbstractPluginConstructorArgs args)
    : InitPtr(args.init_fn_ptr), GetInfoPtr(args.get_info_fn_ptr) {}

void AbstractPlugin::Init(TorustiqHostGlobals globals) {
    ffiPlugin = InitPtr(globals);
}

string AbstractPlugin::GetId() const { return GetInfoPtr().id; }

string AbstractPlugin::GetName() const { return GetInfoPtr().name; }
#include <spdlog/spdlog.h>
void AbstractPlugin::createNewStage(TorustiqPluginStageHandle stageHandle,
                                    TorustiqPluginStageKind stageKind) {
    spdlog::debug("-- Creating new stage with handle {}", stageHandle);
    ffiPlugin.fn_stage_create_new(CreateNewStageFnArgs{
        .stageHandle = stageHandle,
        .stageKind = stageKind,
    });
}

void AbstractPlugin::setConfigValue(TorustiqPluginStageHandle stageHandle,
                                    const char* key, const char* value) {
    ffiPlugin.fn_stage_set_config_value(stageHandle, key, value);
}
