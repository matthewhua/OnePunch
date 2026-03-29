#include <stdio.h>
#include <stdlib.h>

// 二叉树节点结构
typedef struct BinTNode {
    int data;
    struct BinTNode *left;
    struct BinTNode *right;
} BinTNode;

typedef BinTNode* BTree;

// 栈结构定义
typedef struct {
    BTree data[100];
    int top;
} SeqBTreeStack;

// 栈操作函数声明
void initStack(SeqBTreeStack *s);
int push(SeqBTreeStack *s, BTree node);
int pop(SeqBTreeStack *s, BinTNode *node);
int isEmptyS(SeqBTreeStack *s);

void PreorderTraverseNonRecursive(BTree root)
{
    SeqBTreeStack S;
    BinTNode temp;
    
    initStack(&S);
    
    while (1) {
        while (root != NULL) {
            printf("%d \t", root->data);
            if (root->right != NULL) 
                push(&S, root->right);
            root = root->left;
        }
        
        if (isEmptyS(&S) == TRUE) 
            return;
        else {
            pop(&S, &temp);
            root = &temp;
        }
    }
}
