#include "wrapper.h"
#include <iostream>
#include <cstring>  // 添加这行来使用 strcpy


using namespace ogdf;

extern "C" {

void* init_layout() {
    FMMMLayout* layout = new FMMMLayout();
    // layout->useHighLevelOptions(true);
    layout->randSeed(clock());
    layout->useHighLevelOptions(false);
    layout->initialPlacementForces(FMMMLayout::ipfRandomRandIterNr);
    layout->unitEdgeLength(1.0);
    layout->allowedPositions(FMMMLayout::apAll);
    // m_fmmm->minDistCC(m_graphLayoutComponentSeparation);
    layout->stepsForRotatingComponents(50); // Helps to make linear graph components more horizontal.

    layout->initialPlacementForces(ogdf::FMMMLayout::ipfKeepPositions);
    // layout->initialPlacementForces(ogdf::FMMMLayout::ipfRandomTime);
    layout->fixedIterations(120);
    layout->fineTuningIterations(20);
    layout->nmPrecision(8);
    return static_cast<void*>(layout);
}

// allocate a new string and copy the content of the input string
char* allocate_string(const std::string& str) {
    char* cstr = new char[str.length() + 1];
    ::strcpy(cstr, str.c_str());
    return cstr;
}

char* run_layout(const char* input) {
    Graph G;
    GraphAttributes GA(G, GraphAttributes::nodeGraphics | GraphAttributes::edgeGraphics | GraphAttributes::nodeLabel);

    // Convert input string to stream
    std::istringstream iss(input);
    if (!G.readGML(iss)) {
        std::cerr << "Could not parse graph from input" << std::endl;
        return allocate_string("");
    }

    // set weights and heights
    // for (node v : G.nodes)
    //     GA.width(v) = GA.height(v) = 5.0;

    // init layout use init_layout
    FMMMLayout* layout = static_cast<FMMMLayout*>(init_layout());

    // run layout
    layout->call(GA);

    // Write to string stream instead of stdout
    std::ostringstream oss;
    GA.writeGML(oss); // write to string stream

    delete layout;
    return allocate_string(oss.str());
}

void destroy_layout(void* layout) {
    delete static_cast<FMMMLayout*>(layout);
}

// 添加一个函数来释放字符串内存
void free_string(char* str) {
    delete[] str;
}

}
