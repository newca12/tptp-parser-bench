#include <fstream>
#include <iostream>
#include "Parse/TPTP.hpp"
#include "Kernel/Unit.hpp"

extern "C" int parse_tptp_file(const char* path) {
    std::ifstream input(path);
    if (!input.is_open()) {
        return 2;
    }

    try {
        Parse::TPTP parser(input);
        parser.parse();
        return 0;
    } catch (Lib::Exception& e) {
        std::cerr << "Lib::Exception: ";
        e.cry(std::cerr);
        std::cerr << std::endl;
        return 1;
    } catch (...) {
        std::cerr << "Unknown exception" << std::endl;
        return 1;
    }
}

extern "C" int parse_tptp_file_counted(const char* path, int* formula_count) {
    *formula_count = 0;
    std::ifstream input(path);
    if (!input.is_open()) {
        return 2;
    }

    try {
        Parse::TPTP parser(input);
        parser.parse();
        *formula_count = static_cast<int>(UnitList::length(parser.units()));
        return 0;
    } catch (Lib::Exception& e) {
        std::cerr << "Lib::Exception: ";
        e.cry(std::cerr);
        std::cerr << std::endl;
        return 1;
    } catch (...) {
        std::cerr << "Unknown exception" << std::endl;
        return 1;
    }
}
