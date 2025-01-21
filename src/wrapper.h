#ifndef WRAPPER_H
#define WRAPPER_H

#include "../ogdf/energybased/FMMMLayout.h"
#include "../ogdf/basic/Graph.h"
#include "../ogdf/basic/GraphAttributes.h"
#include <string>
#include <cstring>

extern "C" {

// init layout object
void* init_layout();

// run layout
char* run_layout(const char* input);

// destroy layout object
void destroy_layout(void* layout);

void free_string(char* str);

}
#endif
