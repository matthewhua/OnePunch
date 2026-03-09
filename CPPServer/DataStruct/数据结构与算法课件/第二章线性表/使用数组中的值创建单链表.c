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

int createMyListFromArray(LinkList *head, int *a, int n)
    // 读取数组中元素的值，构造一个带头结点的单链表
{
    int i;
    if (n > 0) {
        for (i = 0; i < n; i++) {
            if (insertList(head, i, a[i]) == FALSE) 
                return FALSE;
        }
    }
    return TRUE;
}
