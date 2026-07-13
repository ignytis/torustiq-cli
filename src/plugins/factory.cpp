#include "factory.hpp"

#include <spdlog/spdlog.h>

#include <filesystem>
#include <string>
#include <vector>

#include "../system/dll.hpp"
#include "builtin/file/file.hpp"
#include "builtin/lua/lua.hpp"
#include "builtin/stdio/stdio.hpp"
#include "stage_plugin.hpp"

using namespace std;

using TorustiqCli::Plugins::StagePlugin;
using TorustiqCli::Plugins::StagePluginConstructorArgs;
using TorustiqCli::System::DynamicLibrary;
using TorustiqCli::System::kLibFileExtension;

static StagePlugin BUILTIN_PLUGINS[] = {
    StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
        .init_fn_ptr = TorustiqCli::Plugins::Builtin::File::InitPlugin,
        .get_info_fn_ptr = TorustiqCli::Plugins::Builtin::File::GetPluginInfo,
    }}),
    StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
        .init_fn_ptr = TorustiqCli::Plugins::Builtin::Stdio::InitPlugin,
        .get_info_fn_ptr = TorustiqCli::Plugins::Builtin::Stdio::GetPluginInfo,
    }}),
    StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
        .init_fn_ptr = TorustiqCli::Plugins::Builtin::Lua::InitPlugin,
        .get_info_fn_ptr = TorustiqCli::Plugins::Builtin::Lua::GetPluginInfo,
    }}),
};

vector<StagePlugin> TorustiqCli::Plugins::LoadPlugins(string moduleDir) {
    vector<StagePlugin> plugins(begin(BUILTIN_PLUGINS), end(BUILTIN_PLUGINS));

    for (const filesystem::directory_entry& entry :
         filesystem::directory_iterator(moduleDir)) {
        if (entry.path().extension() != kLibFileExtension) {
            continue;
        }

        // TODO: add error handling (library not found, symbol not found, etc)
        // TODO2: deallocate pointer to dynamic library somewhere
        DynamicLibrary* lib = new DynamicLibrary(entry.path().string());
        TorustiqPluginGetInfoFnPtr getInfoFn =
            lib->get<TorustiqPluginGetInfoFnPtr>("torustiq_plugin_get_info");
        TorustiqPluginInfo pluginInfo = getInfoFn();

        plugins.push_back(
            StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
                .init_fn_ptr =
                    lib->get<TorustiqPluginInitFnPtr>("torustiq_plugin_init"),
                .get_info_fn_ptr = lib->get<TorustiqPluginGetInfoFnPtr>(
                    "torustiq_plugin_get_info"),
            }}));
        spdlog::debug("Loaded library: {}", entry.path().string());
    }

    return plugins;
}
