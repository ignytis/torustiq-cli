#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_ABSTRACT_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_ABSTRACT_STAGE_H_

#include <string>

#include "../../common/collections/tsqueue.hpp"
#include "../../plugins/stage_plugin.hpp"
#include "../../typedefs/pipeline/message.h"
#include "../../typedefs/pipeline/stage.hpp"

using TorustiqCli::Common::Collections::TSQueue;
using TorustiqCli::Plugins::StagePlugin;
using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {

/** Abstract base class for all pipeline stages */
class AbstractStage {
   public:
    explicit AbstractStage(const PipelineStageDefinition& def);
    virtual ~AbstractStage() = default;

    /** Initializes a stage */
    void Init();
    string GetHandlerId() const;
    string GetName() const;

    void SetPlugin(StagePlugin* plugin);
    void Start();

   protected:
    string handlerId;
    string name;

    StagePlugin* plugin = nullptr;
    ConfigKV config;
};

}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_ABSTRACT_STAGE_H_
