#include <fstream>
#include <iostream>
#include <filesystem>
#include "Parse/TPTP.hpp"

extern "C" int parse_tptp_file(const char* path) {
    const auto orig_cwd = std::filesystem::current_path();
    try {
        const auto input_path = std::filesystem::absolute(path);
        // Vampire resolves include directives relative to its working
        // directory when it is given a stream without a source filename.
        std::filesystem::current_path(input_path.parent_path());
        std::ifstream input(input_path);
        if (!input.is_open()) {
            std::filesystem::current_path(orig_cwd);
            return 2;
        }

        Parse::TPTP parser(input);
        parser.parse();
        std::filesystem::current_path(orig_cwd);
        return 0;
    } catch (Lib::Exception& e) {
        std::filesystem::current_path(orig_cwd);
        std::cerr << "Lib::Exception: ";
        e.cry(std::cerr);
        std::cerr << std::endl;
        return 1;
    } catch (...) {
        std::filesystem::current_path(orig_cwd);
        std::cerr << "Unknown exception" << std::endl;
        return 1;
    }
}
