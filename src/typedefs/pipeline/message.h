#ifndef _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_MESSAGE_H_
#define _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_MESSAGE_H_

/**
 * @brief The kind of message that can be sent through the pipeline.
 *  - TORUSTIQ_MESSAGE_KIND_PAYLOAD: A message containing a payload.
 *  - TORUSTIQ_MESSAGE_KIND_TERMINATE: A message indicating that the pipeline should terminate.
 */
enum TorustiqMessageKind {
    TORUSTIQ_MESSAGE_KIND_PAYLOAD,
    TORUSTIQ_MESSAGE_KIND_TERMINATE
};

/**
 * @brief A message that can be sent through the pipeline.
 */
struct TorustiqMessage {
    TorustiqMessageKind kind;
    unsigned int length;
    char* payload;
};

#endif