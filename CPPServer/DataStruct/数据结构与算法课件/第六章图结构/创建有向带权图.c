#include <stdio.h>
#include <stdlib.h>

#define MaxVtxNum 100
#define NB 9999  // 表示不相邻

typedef char VType;

// 图的邻接矩阵结构
typedef struct {
    VType verticesList[MaxVtxNum];
    int AdjMatrix[MaxVtxNum][MaxVtxNum];
    int numVertices;
    int numEdges;
} MGraph;

// 辅助函数声明
int getNumVertices(MGraph g);
int getNumEdges(MGraph g);
int VerToNum(MGraph g, VType v);

int CreateGraph(MGraph *g)  // 创建带权有向图
{
    int i, j, k, w;
    VType u, v, temp;
    
    printf("请输入图的顶点数及边数:");
    scanf("%d %d", &g->numVertices, &g->numEdges);
    printf("请输入图的顶点信息\n");
    scanf("%c", &temp);
    
    for (i = 0; i < getNumVertices(*g); i++)
        scanf("%c", &g->verticesList[i]);
    
    for (i = 0; i < getNumVertices(*g); i++)
        for (j = 0; j < getNumVertices(*g); j++)
            g->AdjMatrix[i][j] = NB;
    
    printf("请输入图的边信息，顶点1 顶点2 权值：\n");
    for (k = 0; k < getNumEdges(*g); k++) {
        scanf("%c", &temp);
        scanf("%c %c %d", &u, &v, &w);
        i = VerToNum(*g, u);
        j = VerToNum(*g, v);
        if (i != -1 && j != -1) 
            g->AdjMatrix[i][j] = w;
    }
    
    return 0;
}
