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

int reverse(LinkList *head)  // 将单链表逆置
{
    LinkNode *left, *middle, *right;
    
    if (*head == NULL) {
        printf("链表错误\n");
        return FALSE;
    }
    
    left = (*head)->next;
    if (left != NULL) 
        middle = left->next;
    left->next = NULL;
    
    while (middle != NULL) {
        right = middle->next;
        middle->next = left;
        left = middle;
        middle = right;
    }
    
    (*head)->next = left;
    return TRUE;
}
