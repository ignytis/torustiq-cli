#include "strings.hpp"

using namespace std;
using namespace TorustiqCli::Common::Strings;

string TorustiqCli::Common::Strings::replaceAll(string str, const string& from,
                                                const string& to) {
    size_t start_pos{0};
    while ((start_pos = str.find(from, start_pos)) != string::npos) {
        str.replace(start_pos, from.length(), to);
        start_pos += to.length();
    }
    return str;
}
