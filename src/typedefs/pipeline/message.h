#ifndef _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_MESSAGE_H_
#define _TORUSTIQ_CLI_TYPEDEFS_PIPELINE_MESSAGE_H_

struct Message {
    unsigned int length;
    char* payload;
};

#endif