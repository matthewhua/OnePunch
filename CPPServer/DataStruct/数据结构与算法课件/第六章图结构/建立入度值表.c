#include <stdio.h>
#include <stdlib.h>

#define MaxVtxNum 100
#define TRUE 1

typedef char VType;

// 图的邻接矩阵结构
typedef struct {
    VType verticesList[MaxVtxNum];
    int AdjMatrix[MaxVtxNum][MaxVtxNum];
    int numVertices;
    int numEdges;
} MGraph;

// 入度表结构
typedef struct {
    VType ver;
    int indeg;
} IndegreeV;

int CreateIndegrTable(MGraph g, IndegreeV *indegreeTable, int *first)
{
    int i, j;
    
    for (i = 0; i < g.numVertices; i++) {
        indegreeTable[i].indeg = 0;
        indegreeTable[i].ver = g.verticesList[i];
        for (j = 0; j < g.numVertices; j++)
            indegreeTable[i].indeg += g.AdjMatrix[j][i];
    }
    
    *first = -1;
    for (j = 0; j < g.numVertices; j++) {
        if (indegreeTable[j].indeg == 0) {
            indegreeTable[j].indeg = *first;
            *first = j;
        }
    }
    
    return TRUE;
}
