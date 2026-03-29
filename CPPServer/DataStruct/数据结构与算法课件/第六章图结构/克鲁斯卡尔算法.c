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
int getNumVertices(MGraph g);
int VerToNum(MGraph g, VType v);
void edgeSort(MGraph g, MSTEN *edge);

void Kruskal(MGraph g, MinST *T)  // Kruskal求MST
{
    int i, n, temp;
    int k = 0, k1 = 0;
    MSTEN *edge;  // 保存边的临时数组
    int *flag, end1, end2;  // 用于标识连通分量
    
    int edgecount = g.numEdges;  // 实际的边数
    n = getNumVertices(g);  // 实际的顶点数
    
    edge = (MSTEN *)malloc(edgecount * sizeof(MSTEN));
    flag = (int *)malloc(n * sizeof(int));
    
    for (i = 0; i < n; i++) 
        flag[i] = i;  // 初始时自成连通分量
    
    edgeSort(g, edge);  // 边排序
    k = 0;  // 记录选中的边数，需要选够n-1条边
    T->totalcost = 0;
    
    for (i = 0; i < edgecount && k < n - 1; i++) {
        end1 = VerToNum(g, edge[i].ver1);
        end2 = VerToNum(g, edge[i].ver2);
        
        if (flag[end1] != flag[end2]) {  // 当前边的两个顶点不在同一连通分量上
            T->edgeArray[k].ver1 = edge[i].ver1;  // 选中该边
            T->edgeArray[k].ver2 = edge[i].ver2;
            T->edgeArray[k].key = edge[i].key;
            T->totalcost += T->edgeArray[k].key;
            k++;
            temp = flag[end2];  // 合并连通分量
            for (k1 = 0; k1 < n; k1++)
                if (flag[k1] == temp) 
                    flag[k1] = flag[end1];
        }
    }
    
    for (i = 0; i < k; i++)
        printf("%c  %c  %d\n", T->edgeArray[i].ver1, T->edgeArray[i].ver2, T->edgeArray[i].key);
    printf("MST = %d\n", T->totalcost);
    
    free(edge);
    free(flag);
}
