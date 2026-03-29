#include <stdio.h>
#include <stdlib.h>

// 链表节点结构
typedef struct LinkNode {
    int data;
    struct LinkNode *next;
} LinkNode;

typedef LinkNode* LinkList;

LinkNode *findKth(LinkList *head, int k)  // 查找倒数第k个结点
{
    LinkNode *front, *rear;
    int i, flag = 1;
    
    if (k <= 0) {
        printf("k必须大于零！");
        return NULL;
    }
    if (*head == NULL) {
        printf("链表错误\n");
        return NULL;
    }
    
    front = *head;
    rear = *head;
    
    for (i = 0; i < k; i++) {
        if (front != NULL) 
            front = front->next;
        else {
            flag = 0;
            break;
        }
    }
    
    if (!flag) {
        printf("k值大于表长！");
        return NULL;
    }
    
    while (front != NULL) {
        front = front->next;
        rear = rear->next;
    }
    
    return rear;
}
