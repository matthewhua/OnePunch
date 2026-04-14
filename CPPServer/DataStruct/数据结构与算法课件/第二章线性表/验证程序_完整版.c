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

// 初始化带头结点的单链表
int initList(LinkList *head) {
    *head = (LinkNode*)malloc(sizeof(LinkNode));
    if (*head == NULL) {
        return FALSE;
    }
    (*head)->next = NULL;
    return TRUE;
}

// 显示链表
void display(LinkList *head) {
    LinkNode *p;
    if (*head == NULL) {
        printf("链表为空\n");
        return;
    }
    p = (*head)->next;
    printf("链表元素：");
    while (p != NULL) {
        printf("%d ", p->data);
        p = p->next;
    }
    printf("\n");
}

// 查找值，返回位置（从0开始）
int find(LinkList *head, int value) {
    LinkNode *p;
    int pos = 0;
    if (*head == NULL) {
        return ERROR;
    }
    p = (*head)->next;
    while (p != NULL) {
        if (p->data == value) {
            return pos;
        }
        p = p->next;
        pos++;
    }
    return ERROR;
}

// 在位置pos插入值（从0开始）
int insertmy(LinkList *head, int pos, int value) {
    LinkNode *p, *newNode;
    int i;
    if (*head == NULL) {
        return FALSE;
    }
    p = *head;
    for (i = 0; i < pos && p != NULL; i++) {
        p = p->next;
    }
    if (p == NULL) {
        return FALSE;
    }
    newNode = (LinkNode*)malloc(sizeof(LinkNode));
    if (newNode == NULL) {
        return FALSE;
    }
    newNode->data = value;
    newNode->next = p->next;
    p->next = newNode;
    return TRUE;
}

// 删除位置pos的值（从0开始）
int removemy(LinkList *head, int pos, int *value) {
    LinkNode *p, *q;
    int i;
    if (*head == NULL || (*head)->next == NULL) {
        return FALSE;
    }
    p = *head;
    for (i = 0; i < pos && p->next != NULL; i++) {
        p = p->next;
    }
    if (p->next == NULL) {
        return FALSE;
    }
    q = p->next;
    *value = q->data;
    p->next = q->next;
    free(q);
    return TRUE;
}

// 查找倒数第k个结点
LinkNode *findKth(LinkList *head, int k) {
    LinkNode *front, *rear;
    int i, flag = 1;
    
    if (k <= 0) {
        printf("k必须大于零！\n");
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
        printf("k值大于表长！\n");
        return NULL;
    }
    
    while (front != NULL) {
        front = front->next;
        rear = rear->next;
    }
    
    return rear;
}

// 查找中间结点
LinkNode *findMiddle(LinkList *head) {
    LinkNode *front, *rear;
    
    if (*head == NULL) {
        printf("链表错误\n");
        return NULL;
    }
    
    front = *head;
    rear = *head;
    
    while (front != NULL) {
        front = front->next;
        if (front != NULL) {
            front = front->next;
            rear = rear->next;
        } else {
            break;
        }
    }
    
    return rear;
}

// 将单链表逆置
int reverse(LinkList *head) {
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

// 从输入创建链表
int createMyList(LinkList *head) {
    int i, counter, k;
    printf("请输入单链表元素个数：");
    scanf("%d", &counter);
    if (counter > 0) {
        printf("请输入构成链表的%d个整数：", counter);
        for (i = 0; i < counter; i++) {
            scanf("%d", &k);
            if (insertmy(head, i, k) == FALSE) 
                return FALSE;
        }
    }
    display(head);
    return TRUE;
}

int main(int argc, char **argv) {
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