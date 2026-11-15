#include <fstream>
#include <iostream>
#include <exception>
#include <filesystem>
#include <system_error>
#include "Parse/TPTP.hpp"

namespace {

bool restore_working_directory(
    const std::filesystem::path& original_cwd,
    bool have_original_cwd,
    bool& changed_cwd
) noexcept {
    if (!have_original_cwd || !changed_cwd) {
        return true;
    }

    try {
        std::error_code error;
        std::filesystem::current_path(original_cwd, error);
        if (error) {
            std::cerr << "failed to restore working directory: " << error.message() << std::endl;
            return false;
        }
    } catch (...) {
        return false;
    }

    changed_cwd = false;
    return true;
}

} // namespace

extern "C" int parse_tptp_file(const char* path) noexcept {
    if (path == nullptr) {
        try {
            std::cerr << "input path is null" << std::endl;
        } catch (...) {
        }
        return 2;
    }

    std::filesystem::path original_cwd;
    bool have_original_cwd = false;
    bool changed_cwd = false;

    try {
        original_cwd = std::filesystem::current_path();
        have_original_cwd = true;
        const auto input_path = std::filesystem::absolute(path);
        // Vampire resolves include directives relative to its working
        // directory when it is given a stream without a source filename.
        std::filesystem::current_path(input_path.parent_path());
        changed_cwd = true;

        std::ifstream input(input_path);
        if (!input.is_open()) {
            restore_working_directory(original_cwd, have_original_cwd, changed_cwd);
            return 2;
        }

        Parse::TPTP parser(input);
        parser.parse();
        if (!restore_working_directory(original_cwd, have_original_cwd, changed_cwd)) {
            return 2;
        }
        return 0;
    } catch (Lib::Exception& e) {
        try {
            std::cerr << "Lib::Exception: ";
            e.cry(std::cerr);
            std::cerr << std::endl;
        } catch (...) {
        }
        restore_working_directory(original_cwd, have_original_cwd, changed_cwd);
        return 1;
    } catch (const std::exception& e) {
        try {
            std::cerr << "Exception: " << e.what() << std::endl;
        } catch (...) {
        }
        restore_working_directory(original_cwd, have_original_cwd, changed_cwd);
        return 1;
    } catch (...) {
        try {
            std::cerr << "Unknown exception" << std::endl;
        } catch (...) {
        }
        restore_working_directory(original_cwd, have_original_cwd, changed_cwd);
        return 1;
    }
}
