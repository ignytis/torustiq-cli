#ifndef _TORUSTIQ_CLI_PLUGINS_BUILTIN_FILE_TYPEDEFS_H_
#define _TORUSTIQ_CLI_PLUGINS_BUILTIN_FILE_TYPEDEFS_H_

#include <torustiq_sdk/typedefs.h>

#include <string>

using namespace std;

namespace TorustiqCli {
namespace Plugins {
namespace Builtin {
namespace File {

enum StageInstanceContentsMode {
    /**
     * Each record is a line in single file.
     */
    FILE_LINES,
    /**
     * Each record is file contents inside directory
     */
    DIRECTORY_FILES,
};

enum StageInstanceMode {
    READER,
    WRITER,
};

/**
 * Represents an instance of a file stage.
 */
class StageInstance {
   public:
    StageInstance(TorustiqPluginStageHandle stageHandle, StageInstanceMode mode,
                  TorustiqHostGlobals& hostGlobals)
        : stageHandle(stageHandle), mode(mode), hostGlobals(hostGlobals) {}

    // Constructor settings
    StageInstanceMode mode = READER;
    TorustiqPluginStageHandle stageHandle;
    TorustiqHostGlobals& hostGlobals;

    // Runtime settings
    StageInstanceContentsMode contentsMode = FILE_LINES;
    string path;
};

}  // namespace File
}  // namespace Builtin
}  // namespace Plugins
}  // namespace TorustiqCli

#endif
