#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0

// 链表节点结构
typedef struct LinkNode {
    int data;
    struct LinkNode *next;
} LinkNode;

typedef LinkNode* LinkList;

// 函数声明
int insertList(LinkList *head, int pos, int value);
void display(LinkList *head);

int createMyList(LinkList *head)  // 构造一个带头结点的单链表
{
    int i, counter, k;
    printf("请输入单链表元素个数：");
    scanf("%d", &counter);
    if (counter > 0) {
        printf("请输入构成链表的%d个整数：", counter);
        for (i = 0; i < counter; i++) {
            scanf("%d", &k);
            if (insertList(head, i, k) == FALSE) 
                return FALSE;
        }
    }
    display(head);
    return TRUE;
}
