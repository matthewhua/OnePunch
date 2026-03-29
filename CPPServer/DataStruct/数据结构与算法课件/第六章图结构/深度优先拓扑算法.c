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

// 辅助函数声明
int getNumVertices(MGraph g);
int VerToNum(MGraph g, VType v);
VType NumToVer(MGraph g, int num);
VType FirstNeighbor(MGraph g, VType v);
VType NextNeighbor(MGraph g, VType v, VType w);

void TopoDFS1(MGraph g, VType v, int *visited)
{
    VType w;
    
    visited[VerToNum(g, v)] = 1;  // 将visited[v]置为1，表示已经访问过
    w = FirstNeighbor(g, v);  // 取v的第一个邻接顶点w
    
    while (w != '#') {
        if (!visited[VerToNum(g, w)])  // 若顶点w还未被访问
            TopoDFS1(g, w, visited);  // 递归调用
        w = NextNeighbor(g, v, w);  // 取下一个邻接顶点
    }
    
    printf("%c ", v);  // 访问顶点v
}

void TopoDFS(MGraph g)
{
    int number, i;
    int visited[MaxVtxNum];  // 定义visited[]
    
    number = getNumVertices(g);
    
    for (i = 0; i < number; i++)  // 初始化visited[]
        visited[i] = 0;
    
    for (i = 0; i < number; i++)  // 打印visited[]
        printf("%d ", visited[i]);
    printf("\n");
    
    for (i = 0; i < number; i++)
        if (visited[i] == 0)
            TopoDFS1(g, NumToVer(g, i), visited);
}
