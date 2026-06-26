#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_PROCESSOR_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_PROCESSOR_STAGE_H_

#include "receiver_stage.hpp"
#include "sender_stage.hpp"

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {

/** A processor stage: accepts data from the previous stage, transforms it, and
 * passes it further */
class ProcessorStage : public virtual ReceiverStage,
                       public virtual SenderStage {
   public:
    explicit ProcessorStage(
        const TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition& def);

    virtual TorustiqPluginStageKind GetStageKind();
};

}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_PROCESSOR_STAGE_H_
