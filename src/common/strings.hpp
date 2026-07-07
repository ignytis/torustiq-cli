#ifndef _TORUSTIQ_CLI_COMMON_STRINGS_H_
#define _TORUSTIQ_CLI_COMMON_STRINGS_H_

#include <string>

using namespace std;

namespace TorustiqCli {
namespace Common {
namespace Strings {

/**
 * Replaces all occurrences of a substring in a string with another substring.
 * Credits:
 * https://www.studyplan.dev/pro-cpp/std-string/q/replace-all-substrings
 */
string replaceAll(std::string str, const std::string& from,
                  const std::string& to);

}  // namespace Strings
}  // namespace Common
}  // namespace TorustiqCli

#endif
