#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0
#define ERROR -1

// 链表节点结构
typedef struct LinkNode {
    int data;
    struct LinkNode *next;
} LinkNode;

typedef LinkNode* LinkList;

// 函数声明
int initList(LinkList *head);
int createMyList(LinkList *head);
void display(LinkList *head);
int find(LinkList *head, int value);
int insertmy(LinkList *head, int pos, int value);
int removemy(LinkList *head, int pos, int *value);
LinkNode *findKth(LinkList *head, int k);
LinkNode *findMiddle(LinkList *head);
int reverse(LinkList *head);

int main(int argc, char **argv)
{
    LinkList head = NULL;
    LinkNode *result;
    int i, k;
    
    i = initList(&head);
    if (i == 0) 
        printf("链表初始化错误\n");
    else 
        printf("初始化完成\n");
    
    createMyList(&head);
    
    printf("请输入要查找的值：");
    scanf("%d", &i);
    k = find(&head, i);
    if (k == ERROR) 
        printf("没有找到 %d\n", i);
    
    insertmy(&head, k, 100);
    printf("插入后的链表，");
    display(&head);
    
    printf("请输入要删除的值：");
    scanf("%d", &i);
    k = find(&head, i);
    removemy(&head, k, &k);
    printf("删除后的链表，");
    display(&head);
    
    printf("请输入要查找的结点的倒数位置：");
    scanf("%d", &k);
    result = findKth(&head, k);
    if (result != NULL) 
        printf("data = %d\n", result->data);
    else 
        printf("给定的参数不正确\n");
    
    result = findMiddle(&head);
    if (result != NULL) 
        printf("middle data = %d\n", result->data);
    else 
        printf("给定的参数不正确\n");
    
    printf("单链表逆置后：\n");
    reverse(&head);
    display(&head);
    
    return 0;
}
