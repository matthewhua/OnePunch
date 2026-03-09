#include <stdio.h>
#include <stdlib.h>

// 链表节点结构
typedef struct LinkNode {
    int data;
    struct LinkNode *next;
} LinkNode;

typedef LinkNode* LinkList;

LinkNode *findMiddle(LinkList *head)  // 查找中间结点，返回指向中间结点的指针
{
    LinkNode *front, *rear;
    
    if (*head == NULL) {
        printf("链表错误\n");
        return NULL;
    }
    
    front = *head;
    rear = *head;
    
    while (front != NULL) {  // 当前指针front没有走到最后一个结点时，继续循环
        front = front->next;
        if (front != NULL) {
            front = front->next;
            rear = rear->next;
        }
        else {
            break;
        }
    }
    
    return rear;
}
