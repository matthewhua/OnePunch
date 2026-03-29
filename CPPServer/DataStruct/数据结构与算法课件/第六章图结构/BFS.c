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

// 队列结构定义
typedef struct {
    VType data[MaxVtxNum];
    int front;
    int rear;
} SeqQueue;

// 辅助函数声明
int getNumVertices(MGraph g);
int VerToNum(MGraph g, VType v);
VType FirstNeighbor(MGraph g, VType v);
VType NextNeighbor(MGraph g, VType v, VType w);
void initQueue(SeqQueue *q);
int enqueue(SeqQueue *q, VType v);
int dequeue(SeqQueue *q, VType *v);
int isEmpty(SeqQueue q);

void BFS(MGraph g, VType v)  // 图的广度优先遍历
{
    int number, i;
    SeqQueue q;  // 初始化队列q
    VType w;
    int visited[MaxVtxNum];  // 定义visited[]
    
    initQueue(&q);  // 初始化队列q
    number = getNumVertices(g);
    
    for (i = 0; i < number; i++) 
        visited[i] = 0;  // 初始化visited[]
    
    printf("%c ", v);  // 访问顶点v
    visited[VerToNum(g, v)] = 1;  // 表示v已经访问过
    enqueue(&q, v);  // 将顶点v入队列
    
    while (!isEmpty(q)) {  // 若q不为空
        dequeue(&q, &v);  // 出队列
        w = FirstNeighbor(g, v);  // 取v的第一个邻接顶点w
        
        while (w != '#') {
            if (!visited[VerToNum(g, w)]) {
                printf("%c ", w);  // 访问顶点w
                visited[VerToNum(g, w)] = 1;  // 表示w已经访问过
                enqueue(&q, w);  // 将w入队列
            }
            w = NextNeighbor(g, v, w);  // 取下一个邻接顶点
        }
    }
}
