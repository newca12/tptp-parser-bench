#include <fstream>
#include <iostream>
#include "Parse/TPTP.hpp"

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
