#ifndef _TORUSTIQ_CLI_PLUGINS_BUILTIN_LUA_H_
#define _TORUSTIQ_CLI_PLUGINS_BUILTIN_LUA_H_

#include <torustiq_sdk/plugins/typedefs.h>

namespace TorustiqCli {
namespace Plugins {
namespace Builtin {
namespace Lua {

const TorustiqPluginInfo GetPluginInfo();

const TorustiqPlugin InitPlugin(TorustiqHostGlobals globals);

void CreateNewStage(CreateNewStageFnArgs args);

void SetStageConfigValue(TorustiqPluginStageHandle stageHandle, const char* key,
                         const char* value);

void Start(TorustiqPluginStageHandle stageHandle);

}  // namespace Lua
}  // namespace Builtin
}  // namespace Plugins
}  // namespace TorustiqCli

#endif
