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

// 辅助数组结构
typedef struct {
    VType vex;
    int lowcost;
} ClosEdge;

// 最小生成树边结构
typedef struct {
    VType ver1;
    VType ver2;
    int key;
} MSTEN;

// 最小生成树结构
typedef struct {
    MSTEN edgeArray[MaxVtxNum];
    int totalcost;
} MinST;

// 辅助函数声明
VType NumToVer(MGraph g, int num);

int Prim(MGraph g, int u, ClosEdge closedge[], MinST *T)
{
    /* 从顶点u开始构造图g的最小生成树，计算过程在辅助数组closedge中，最小生成树保存在T中*/
    int i, j, w, k, count = 0;
    int n = g.numVertices;  // 结点个数
    T->totalcost = 0;
    
    for (i = 0; i < n; i++)  // 辅助数组初始化
        if (i != u) {
            closedge[i].vex = NumToVer(g, u);
            closedge[i].lowcost = g.AdjMatrix[u][i];
        }
    
    closedge[u].lowcost = 0;  // 初始，U={u}
    
    for (i = 0; i < n - 1; i++) {  // 选择其余的n-1个顶点
        w = NB;
        for (j = 0; j < n; j++)  // 在辅助数组closedge中选择权值最小的顶点
            if (closedge[j].lowcost != 0 && closedge[j].lowcost < w) {
                w = closedge[j].lowcost;
                k = j;
            }  // 找到生成树的下一个顶点k
        
        closedge[k].lowcost = 0;  // 第k顶点并入U集
        T->edgeArray[count].ver1 = NumToVer(g, k);
        T->edgeArray[count].ver2 = closedge[k].vex;
        T->edgeArray[count].key = w;
        T->totalcost += w;
        count++;
        
        for (j = 0; j < n; j++)  // 新顶点k并入U后，修改辅助数组
            if (g.AdjMatrix[k][j] < closedge[j].lowcost) {
                closedge[j].vex = NumToVer(g, k);
                closedge[j].lowcost = g.AdjMatrix[k][j];
            }
    }
    
    for (i = 0; i < n - 1; i++)
        printf("%c  %c  %d\n", T->edgeArray[i].ver1, T->edgeArray[i].ver2, T->edgeArray[i].key);
    printf("MST = %d\n", T->totalcost);
    
    return T->totalcost;
}
