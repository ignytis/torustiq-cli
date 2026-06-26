#ifndef _TORUSTIQ_CLI_PLUGINS_STAGE_PLUGIN_H_
#define _TORUSTIQ_CLI_PLUGINS_STAGE_PLUGIN_H_

// #include <torustiq_sdk/plugins/typedefs.h>

#include "abstract_plugin.hpp"

namespace TorustiqCli {
namespace Plugins {

struct StagePluginConstructorArgs {
    AbstractPluginConstructorArgs abstract_plugin_args;
};

/** A class for stage plugin */
class StagePlugin : public AbstractPlugin {
   public:
    StagePlugin(StagePluginConstructorArgs args);
    void Start(TorustiqPluginStageHandle stageHandle);
};
}  // namespace Plugins
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PLUGINS_STAGE_PLUGIN_H_
