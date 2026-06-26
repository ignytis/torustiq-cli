#include "abstract_plugin.hpp"

#include <torustiq_sdk/plugins/typedefs.h>

using namespace TorustiqCli::Plugins;

AbstractPlugin::AbstractPlugin(AbstractPluginConstructorArgs args)
    : InitPtr(args.init_fn_ptr), GetInfoPtr(args.get_info_fn_ptr) {}

void AbstractPlugin::init() { ffiPlugin = InitPtr(); }

string AbstractPlugin::GetId() const { return GetInfoPtr().id; }

string AbstractPlugin::GetName() const { return GetInfoPtr().name; }

TorustiqPluginStageHandle AbstractPlugin::createNewStage(
    TorustiqPluginStageKind stageKind) {
    return ffiPlugin.fn_create_new_stage(CreateNewStageFnArgs{
        .stageKind = stageKind,
    });
}

void AbstractPlugin::setConfigValue(TorustiqPluginStageHandle stageHandle,
                                    const char* key, const char* value) {
    ffiPlugin.fn_set_config_value(stageHandle, key, value);
}
