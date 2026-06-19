#include "factory.hpp"

#include <vector>

#include "../stage_plugin.hpp"
#include "file/file.hpp"
#include "stdio/stdio.hpp"

using namespace std;

using TorustiqCli::Plugins::StagePlugin;

vector<StagePlugin> TorustiqCli::Plugins::Builtin::GetBuiltinPlugins() {
    vector<StagePlugin> plugins;

    plugins.push_back(
        StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
            .init_fn_ptr = TorustiqCli::Plugins::Builtin::File::InitPlugin,
            .get_info_fn_ptr =
                TorustiqCli::Plugins::Builtin::File::GetPluginInfo,
        }}));
    plugins.push_back(
        StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
            .init_fn_ptr = TorustiqCli::Plugins::Builtin::Stdio::InitPlugin,
            .get_info_fn_ptr =
                TorustiqCli::Plugins::Builtin::Stdio::GetPluginInfo,
        }}));

    return plugins;
}
