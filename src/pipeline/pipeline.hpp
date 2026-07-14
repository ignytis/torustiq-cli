#ifndef _TORUSTIQ_CLI_PIPELINE_PIPELINE_H_
#define _TORUSTIQ_CLI_PIPELINE_PIPELINE_H_

#include <torustiq_sdk/typedefs.h>

#include <string>
#include <unordered_set>
#include <vector>

#include "../plugins/stage_plugin.hpp"
#include "../typedefs/pipeline/pipeline.hpp"
#include "stages/abstract_stage.hpp"

namespace TorustiqCli {
namespace Pipeline {

using Typedefs::Pipeline::PipelineDefinition;

/** A pipeline containing a linear sequence of stages: one source, zero or more
 * processors, one sink */
class Pipeline {
   public:
    /** Constructs a pipeline from a pipeline definition, instantiating stages
     * by position: first = source, last = sink, middle = processors.
     * @param module_dir Path to directory containing plugin libraries to load
     */
    Pipeline(const PipelineDefinition& def);

    /** Returns a set of handlers which are in use by pipeline */
    unordered_set<string> getHandlersInUse();

    /** Sets plugins which are used by pipeline. Injects plugins to stages */
    void setPlugins(vector<TorustiqCli::Plugins::StagePlugin>& plugins);

    /**
     * Initializes stages by calling the initialization method in for each stage
     */
    void initStages();

    /**
     * Starts a pipeline
     */
    void start();

    Stages::AbstractStage* GetStagePtrByHandle(
        TorustiqPluginStageHandle handle);

   private:
    vector<Stages::AbstractStage*> stages;
};

}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_PIPELINE_H_
