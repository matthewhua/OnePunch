#include <stdio.h>
#include <stdlib.h>

#define MaxVtxNum 100

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

void Topo(MGraph g, IndegreeV *indegreeTable, int first)
{
    int i, temp;
    
    while (first != -1) {
        printf("%c ", g.verticesList[first]);
        temp = first;
        first = indegreeTable[first].indeg;
        
        for (i = 0; i < g.numVertices; i++) {
            if (g.AdjMatrix[temp][i] > 0) {
                indegreeTable[i].indeg--;
                if (indegreeTable[i].indeg == 0) {
                    indegreeTable[i].indeg = first;
                    first = i;
                }
            }
        }
    }
}
