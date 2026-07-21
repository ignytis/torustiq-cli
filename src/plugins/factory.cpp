#include "factory.hpp"

#include <spdlog/spdlog.h>

#include <filesystem>
#include <memory>
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

static vector<unique_ptr<DynamicLibrary>> loaded_dynamic_libraries;

vector<StagePlugin> TorustiqCli::Plugins::LoadPlugins(string moduleDir) {
    vector<StagePlugin> plugins(begin(BUILTIN_PLUGINS), end(BUILTIN_PLUGINS));

    if (!filesystem::exists(moduleDir)) {
        spdlog::warn("Plugin directory does not exist: {}", moduleDir);
        return plugins;
    }

    if (!filesystem::is_directory(moduleDir)) {
        spdlog::warn("Plugin path is not a directory: {}", moduleDir);
        return plugins;
    }

    for (const filesystem::directory_entry& entry :
         filesystem::directory_iterator(moduleDir)) {
        if (entry.path().extension() != kLibFileExtension) {
            continue;
        }

        const string& lib_path = entry.path().string();
        spdlog::debug("Attempting to load plugin: {}", lib_path);

        // Load library with error handling
        auto lib = make_unique<DynamicLibrary>(lib_path);
        if (!lib->IsValid()) {
            spdlog::error("Failed to load plugin library: {} - {}",
                         lib_path, lib->GetLastError());
            continue;
        }

        // Load GetInfo function
        TorustiqPluginGetInfoFnPtr getInfoFn =
            lib->get<TorustiqPluginGetInfoFnPtr>("torustiq_plugin_get_info");
        if (!getInfoFn) {
            spdlog::error("Failed to load torustiq_plugin_get_info from {}: {}",
                         lib_path, lib->GetLastError());
            continue;
        }

        // Load Init function
        TorustiqPluginInitFnPtr initFn =
            lib->get<TorustiqPluginInitFnPtr>("torustiq_plugin_init");
        if (!initFn) {
            spdlog::error("Failed to load torustiq_plugin_init from {}: {}",
                         lib_path, lib->GetLastError());
            continue;
        }

        // Validate plugin info
        TorustiqPluginInfo pluginInfo = getInfoFn();
        if (!pluginInfo.id || strlen(pluginInfo.id) == 0) {
            spdlog::error("Plugin has invalid/empty ID: {}", lib_path);
            continue;
        }
        if (!pluginInfo.name || strlen(pluginInfo.name) == 0) {
            spdlog::error("Plugin has invalid/empty name: {}", lib_path);
            continue;
        }

        // Add to loaded libraries to keep alive
        loaded_dynamic_libraries.push_back(move(lib));

        // Create and add plugin
        plugins.push_back(
            StagePlugin(StagePluginConstructorArgs{.abstract_plugin_args{
                .init_fn_ptr = initFn,
                .get_info_fn_ptr = getInfoFn,
            }}));

        spdlog::info("Successfully loaded plugin: {} (id: {})",
                    lib_path, pluginInfo.id);
    }

    return plugins;
}
