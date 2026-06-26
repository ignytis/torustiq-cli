#include "stage_plugin.hpp"

using namespace TorustiqCli::Plugins;

StagePlugin::StagePlugin(StagePluginConstructorArgs args)
    : AbstractPlugin(args.abstract_plugin_args) {}

void StagePlugin::Start(TorustiqPluginStageHandle stageHandle) {
    this->ffiPlugin.fn_stage_start(stageHandle);
}
