#ifndef _TORUSTIQ_CLI_PLUGINS_ABSTRACT_PLUGIN_H_
#define _TORUSTIQ_CLI_PLUGINS_ABSTRACT_PLUGIN_H_

#include <torustiq_sdk/plugins/typedefs.h>

#include <string>

using namespace std;

namespace TorustiqCli {
namespace Plugins {

struct AbstractPluginConstructorArgs {
    TorustiqPluginInitFnPtr init_fn_ptr;
    TorustiqPluginGetInfoFnPtr get_info_fn_ptr;
};

/** Abstract base class for plugins */
class AbstractPlugin {
   public:
    AbstractPlugin(AbstractPluginConstructorArgs args);

    void Init(TorustiqHostGlobals globals);

    string GetId() const;
    string GetName() const;

    void createNewStage(TorustiqPluginStageHandle stageHandle,
                        TorustiqPluginStageKind stageKind);
    void setConfigValue(TorustiqPluginStageHandle stageHandle, const char* key,
                        const char* value);

   protected:
    TorustiqPluginInitFnPtr InitPtr;
    TorustiqPluginGetInfoFnPtr GetInfoPtr;

    TorustiqPlugin ffiPlugin;
};

}  // namespace Plugins
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PLUGINS_ABSTRACT_PLUGIN_H_
