#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_PROCESSOR_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_PROCESSOR_STAGE_H_

#include <torustiq_sdk/plugins/typedefs.h>

#include "mixins/receiver_stage.hpp"
#include "mixins/sender_stage.hpp"

using TorustiqCli::Pipeline::Stages::Mixins::ReceiverStage;
using TorustiqCli::Pipeline::Stages::Mixins::SenderStage;

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {

/** A processor stage: accepts data from the previous stage, transforms it, and
 * passes it further */
class ProcessorStage : public virtual ReceiverStage,
                       public virtual SenderStage {
   public:
    explicit ProcessorStage(const PipelineStageDefinition& def,
                            HostGlobals globals);

    virtual TorustiqPluginStageKind GetStageKind();
};

}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_PROCESSOR_STAGE_H_
