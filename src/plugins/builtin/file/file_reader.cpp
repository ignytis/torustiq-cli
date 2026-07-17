#include "file_reader.hpp"

#include <spdlog/spdlog.h>
#include <torustiq_sdk/message.h>

#include <filesystem>
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
}

void startDirectoryFilesReader(StageInstance* instance) {
    TorustiqPluginStageHandle stageHandle = instance->stageHandle;
    for (const filesystem::directory_entry& entry :
         filesystem::directory_iterator(instance->path)) {
        TorustiqMessage msg{};
        ifstream file(entry.path());

        std::ostringstream content;
        content << file.rdbuf();
        string contentStr = content.str();

        msg.type = TORUSTIQ_MESSAGE_TYPE_DATA;
        msg.payload_size = contentStr.size();
        msg.payload = reinterpret_cast<uint8_t*>(contentStr.data());
        instance->hostGlobals.sendMessageFnPtr(stageHandle, &msg);
    }
}

}  // namespace

void TorustiqCli::Plugins::Builtin::File::startReader(StageInstance* instance) {
    TorustiqPluginStageHandle stageHandle = instance->stageHandle;
    switch (instance->contentsMode) {
        case StageInstanceContentsMode::FILE_LINES:
            startFileLinesReader(instance);
            break;
        case StageInstanceContentsMode::DIRECTORY_FILES:
            startDirectoryFilesReader(instance);
            break;
    }

    TorustiqMessage msg = torustiq_message_create_eof();
    instance->hostGlobals.sendMessageFnPtr(stageHandle, &msg);

    spdlog::debug("file :: Reached EOF in reader with handle {}", stageHandle);
}
