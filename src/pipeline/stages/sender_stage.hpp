#ifndef _TORUSTIQ_CLI_PIPELINE_STAGES_SENDER_STAGE_H_
#define _TORUSTIQ_CLI_PIPELINE_STAGES_SENDER_STAGE_H_

#include "abstract_stage.hpp"

using TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition;

namespace TorustiqCli {
namespace Pipeline {
namespace Stages {

/**
 * @brief A sender stage: accepts an incoming payload from the previous stage
 * and forwards it to the next stage.
 */
class SenderStage : virtual public AbstractStage {
   public:
    explicit SenderStage(
        const TorustiqCli::Typedefs::Pipeline::PipelineStageDefinition& def);
    void SetOutputQueuePtr(TSQueue<TorustiqMessage>* queue);

   protected:
    /** The output queue for sending messages */
    TSQueue<TorustiqMessage>* outputQueue = nullptr;
};

}  // namespace Stages
}  // namespace Pipeline
}  // namespace TorustiqCli

#endif  // _TORUSTIQ_CLI_PIPELINE_STAGES_SENDER_STAGE_H_
