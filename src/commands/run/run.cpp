#include "run.hpp"

#include <spdlog/spdlog.h>
#include <yaml-cpp/yaml.h>

#include <filesystem>
#include <ranges>
#include <unordered_set>
#include <vector>

#include "../../pipeline/pipeline.hpp"
#include "../../plugins/builtin/factory.hpp"
#include "../../plugins/stage_plugin.hpp"
#include "../../system/dll.hpp"
#include "../../typedefs/pipeline/pipeline.hpp"

using namespace std;
using namespace std::ranges;

using TorustiqCli::Commands::Run::RunCommand;
using TorustiqCli::Pipeline::Pipeline;
using TorustiqCli::System::DynamicLibrary;
using TorustiqCli::System::kLibFileExtension;
using TorustiqCli::Typedefs::Pipeline::PipelineDefinition;

RunCommand::RunCommand(Configuration* config, string pipeline_path)
    : config(config), pipeline_path(pipeline_path) {}

void RunCommand::run() {
    spdlog::info("Executing a pipeline from file: {}", pipeline_path);

    PipelineDefinition pipeDef =
        YAML::LoadFile(pipeline_path).as<PipelineDefinition>();
    Pipeline::Pipeline pipeline = Pipeline::Pipeline(pipeDef);
    spdlog::debug("Pipeline name: {}", pipeDef.name);

    unordered_set<string> handlers = pipeline.getHandlersInUse();
    spdlog::debug("Pipeline handlers in use:");
    for (const string& handler : handlers) {
        spdlog::debug("- {}", handler);
    }

    // Format a set of plugins. Start with builtins
    vector<StagePlugin> plugins_builtin =
        TorustiqCli::Plugins::Builtin::GetBuiltinPlugins();

    // TODO: implement plugins here
    for (const filesystem::directory_entry& entry :
         filesystem::directory_iterator(config->moduleDir)) {
        if (entry.path().extension() == kLibFileExtension) {
            DynamicLibrary lib = DynamicLibrary(entry.path().string());
            // lib.get("init");
        }
    }

    vector<StagePlugin> plugins =
        plugins_builtin | views::filter([&handlers](const StagePlugin& plugin) {
            return handlers.contains(plugin.GetId());
        }) |
        to<vector<StagePlugin>>();

    spdlog::debug("Initializing plugins...");
    for (StagePlugin& plugin : plugins) {
        plugin.init();
        spdlog::debug("Plugin [{}] ({}) was initialized.", plugin.GetId(),
                      plugin.GetName());
    }
    spdlog::debug("Plugin initialization complete.");

    pipeline.setPlugins(plugins);
    pipeline.initStages();

    pipeline.start();
}
