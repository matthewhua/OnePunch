#include <stdio.h>
#include <stdlib.h>

#define MaxVtxNum 100

// 迷宫路径栈节点
typedef struct {
    int i;
    int j;
    int d;
} StackNode;

StackNode stack[100];

void mazepath(int maze[][6], int m, int n)  // maze是加围墙边的迷宫矩阵
{
    int mark[6][6] = {0};  // 初始化，R进入迷宫
    int move[2][4] = {{0, 1, 0, -1}, {1, 0, -1, 0}};
    int top = -1;
    int i = 1, j = 1, d = 0;
    int g = 0, h = 0;
    
    mark[1][1] = 1;
    
    while ((g != m - 2) || (h != n - 2)) {
        g = i + move[0][d];
        h = j + move[1][d];  // 进行试探
        
        if ((maze[g][h] == 0) && (mark[g][h] == 0)) {
            mark[g][h] = 1;  // 进入新位置
            top = top + 1;
            stack[top].i = i;
            stack[top].j = j;
            stack[top].d = d;
            i = g;
            j = h;
            d = 0;
        }
        else {
            if (d < 3) 
                d = d + 1;  // 换新方向再试探(右、下、左、上)
            else {
                if (top > 0) {  // 后退一步再试探
                    i = stack[top].i;
                    j = stack[top].j;
                    d = stack[top].d;
                    top = top - 1;
                }
                else {
                    printf("此迷宫没有通路!\n");  // 迷宫无通路
                    return;
                }
            }
        }
    }
    
    printf("此迷宫有通路 !\n");  // 走出迷宫
    printf("通路由以下位置构成：\n");
    for (i = 0; i <= top; i++) {
        printf("( %d , %d )", stack[i].i, stack[i].j);
        if ((i + 1) % 10 == 0) {
            printf("\n");
        }
    }
    printf("( %d , %d )\n", g, h);
    printf("\n");
}

int main()
{
    int maze[6][6] = {
        {1, 1, 1, 1, 1, 1},
        {1, 0, 0, 0, 1, 1},
        {1, 0, 1, 0, 0, 1},
        {1, 0, 0, 0, 1, 1},
        {1, 1, 0, 0, 0, 1},
        {1, 1, 1, 1, 1, 1}
    };
    
    printf("迷宫矩阵（0表示通路，1表示墙）：\n");
    for (int i = 0; i < 6; i++) {
        for (int j = 0; j < 6; j++) {
            printf("%d ", maze[i][j]);
        }
        printf("\n");
    }
    printf("\n");
    
    mazepath(maze, 6, 6);
    return 0;
}
