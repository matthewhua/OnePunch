#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0

// 队列结构定义
typedef struct {
    int data[100];
    int front;
    int rear;
} SeqQueue;

// 队列操作函数声明
void initQueue(SeqQueue *q);
int enqueue(SeqQueue *q, int value);
int dequeue(SeqQueue *q, int *value);

void printblank(int n)  // 打印n个空格
{
    int i;
    for (i = 0; i < n; i++) 
        printf(" ");
}

void yangTri(int n)
{
    SeqQueue myq;
    int i, j, k, first, second, add;
    
    initQueue(&myq);
    printblank(n - 1);
    printf("   1\n");  // 直接输出第1行
    enqueue(&myq, 1);  // 第二行的左1入队
    
    for (i = 2; i <= n; i++) {  // 输出第二行至第n行
        enqueue(&myq, 1);  // 下一行左1入队
        first = 1;
        dequeue(&myq, &first);
        printblank(n - i);
        printf("%d ", first);  // 输出本行左1
        
        for (j = 1; j < i - 1; j++) {  // 输出中间部分
            dequeue(&myq, &second);
            add = first + second;
            printf("%d ", add);  // 输出前一行左右值之和
            enqueue(&myq, add);
            first = second;
        }
        
        printf("%d ", 1);  // 输出本行右1
        enqueue(&myq, 1);  // 下一行右1入队
        printf("\n");
    }
}

int main()
{
    int n;
    printf("请输入杨辉三角的行数：");
    scanf("%d", &n);
    yangTri(n);
    return 0;
}
