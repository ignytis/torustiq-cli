#ifndef _TORUSTIQ_CLI_PLUGINS_BUILTIN_FILE_H_
#define _TORUSTIQ_CLI_PLUGINS_BUILTIN_FILE_H_

#include <torustiq_sdk/plugins/typedefs.h>

namespace TorustiqCli {
namespace Plugins {
namespace Builtin {
namespace File {

const TorustiqPluginInfo GetPluginInfo();

const TorustiqPlugin InitPlugin();

TorustiqPluginStageHandle CreateNewStage();

void SetConfigValue(TorustiqPluginStageHandle stageHandle, const char* key,
                    const char* value);

}  // namespace File
}  // namespace Builtin
}  // namespace Plugins
}  // namespace TorustiqCli

#endif
