#include "file_reader.hpp"

#include <spdlog/spdlog.h>
#include <torustiq_sdk/message.h>

#include <fstream>

#include "typedefs.hpp"

using namespace std;
using namespace TorustiqCli::Plugins::Builtin::File;

namespace {

void startFileLinesReader(StageInstance* instance) {
    // Open file for reading
    TorustiqPluginStageHandle stageHandle = instance->stageHandle;

    spdlog::debug("file :: Starting reader stage with handle {}", stageHandle);
    ifstream file(instance->path);
    string line;
    while (getline(file, line)) {
        TorustiqMessage msg{};
        msg.type = TORUSTIQ_MESSAGE_TYPE_DATA;
        msg.payload_size = line.size();
        msg.payload = reinterpret_cast<uint8_t*>(line.data());
        instance->hostGlobals.sendMessageFnPtr(stageHandle, &msg);
    }

    TorustiqMessage msg = torustiq_message_create_eof();
    instance->hostGlobals.sendMessageFnPtr(stageHandle, &msg);

    spdlog::debug("file :: Reached EOF in reader with handle {}", stageHandle);
}

void startDirectoryFilesReader(StageInstance* instance) {
    spdlog::error("Directory files mode is not yet implemented");
    exit(EXIT_FAILURE);
}

}  // namespace

void TorustiqCli::Plugins::Builtin::File::startReader(StageInstance* instance) {
    switch (instance->contentsMode) {
        case StageInstanceContentsMode::FILE_LINES:
            startFileLinesReader(instance);
        case StageInstanceContentsMode::DIRECTORY_FILES:
            startDirectoryFilesReader(instance);
    }
}
