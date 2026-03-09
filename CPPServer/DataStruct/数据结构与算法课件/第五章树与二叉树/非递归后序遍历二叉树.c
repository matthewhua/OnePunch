#include <stdio.h>
#include <stdlib.h>

#define TRUE 1

// 二叉树节点结构
typedef struct BinTNode {
    int data;
    struct BinTNode *left;
    struct BinTNode *right;
} BinTNode;

typedef BinTNode* BTree;

// 后序遍历栈节点
typedef struct {
    BinTNode btreeNode;
    int flag;
} StackTNode;

// 栈结构定义
typedef struct {
    StackTNode data[100];
    int top;
} SeqBTreeStackforPostT;

// 栈操作函数声明
void initStack(SeqBTreeStackforPostT *s);
int push(SeqBTreeStackforPostT *s, StackTNode *node);
int pop(SeqBTreeStackforPostT *s, StackTNode *node);
int isEmpty(SeqBTreeStackforPostT *s);

void PostorderTraverseNonRecursive(BTree root)
{
    SeqBTreeStackforPostT S;
    StackTNode stacktemp;
    
    initStack(&S);
    
    while (1) {
        while (root != NULL) {
            stacktemp.btreeNode = *root;
            stacktemp.flag = 0;
            push(&S, &stacktemp);
            root = root->left;
        }
        
        if (isEmpty(&S) == TRUE) 
            return;
        else {
            pop(&S, &stacktemp);
            if (stacktemp.flag == 0) {
                stacktemp.flag = 1;
                push(&S, &stacktemp);
                root = stacktemp.btreeNode.right;
            }
            else {
                printf("%d \t", stacktemp.btreeNode.data);
            }
        }
    }
}
