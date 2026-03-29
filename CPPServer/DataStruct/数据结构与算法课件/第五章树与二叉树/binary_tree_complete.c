#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0

// 二叉树节点结构
typedef struct BinTNode {
    int data;
    struct BinTNode *left;
    struct BinTNode *right;
} BinTNode;

typedef BinTNode* BTree;

// 栈结构定义（用于非递归遍历）
typedef struct {
    BTree data[100];
    int top;
} SeqBTreeStack;

// 后序遍历栈节点
typedef struct {
    BinTNode btreeNode;
    int flag;
} StackTNode;

typedef struct {
    StackTNode data[100];
    int top;
} SeqBTreeStackforPostT;

// 栈操作函数
void initStack(SeqBTreeStack *s) {
    s->top = -1;
}

int push(SeqBTreeStack *s, BTree node) {
    if (s->top >= 99) return FALSE;
    s->data[++(s->top)] = node;
    return TRUE;
}

int pop(SeqBTreeStack *s, BTree *node) {
    if (s->top < 0) return FALSE;
    *node = s->data[(s->top)--];
    return TRUE;
}

int isEmptyS(SeqBTreeStack *s) {
    return s->top < 0;
}

// 后序遍历栈操作函数
void initStackPost(SeqBTreeStackforPostT *s) {
    s->top = -1;
}

int pushPost(SeqBTreeStackforPostT *s, StackTNode *node) {
    if (s->top >= 99) return FALSE;
    s->data[++(s->top)] = *node;
    return TRUE;
}

int popPost(SeqBTreeStackforPostT *s, StackTNode *node) {
    if (s->top < 0) return FALSE;
    *node = s->data[(s->top)--];
    return TRUE;
}

int isEmptyPost(SeqBTreeStackforPostT *s) {
    return s->top < 0;
}

// 创建新节点
BTree createNode(int data) {
    BTree newNode = (BTree)malloc(sizeof(BinTNode));
    if (newNode == NULL) {
        printf("内存分配失败\n");
        return NULL;
    }
    newNode->data = data;
    newNode->left = NULL;
    newNode->right = NULL;
    return newNode;
}

// 递归先序遍历
void PreorderTraverse(BTree root) {
    if (root != NULL) {
        printf("%d ", root->data);
        PreorderTraverse(root->left);
        PreorderTraverse(root->right);
    }
}

// 递归中序遍历
void InorderTraverse(BTree root) {
    if (root != NULL) {
        InorderTraverse(root->left);
        printf("%d ", root->data);
        InorderTraverse(root->right);
    }
}

// 递归后序遍历
void PostorderTraverse(BTree root) {
    if (root != NULL) {
        PostorderTraverse(root->left);
        PostorderTraverse(root->right);
        printf("%d ", root->data);
    }
}

// 非递归先序遍历
void PreorderTraverseNonRecursive(BTree root) {
    SeqBTreeStack S;
    BTree temp;
    
    initStack(&S);
    
    while (1) {
        while (root != NULL) {
            printf("%d ", root->data);
            if (root->right != NULL) 
                push(&S, root->right);
            root = root->left;
        }
        
        if (isEmptyS(&S) == TRUE) 
            return;
        else {
            pop(&S, &temp);
            root = temp;
        }
    }
}

// 非递归中序遍历
void InorderTraverseNonRecursive(BTree root) {
    SeqBTreeStack S;
    BTree temp;
    
    initStack(&S);
    
    while (1) {
        while (root != NULL) {
            push(&S, root);
            root = root->left;
        }
        
        if (isEmptyS(&S) == TRUE) 
            return;
        else {
            pop(&S, &temp);
            printf("%d ", temp->data);
            root = temp->right;
        }
    }
}

// 非递归后序遍历
void PostorderTraverseNonRecursive(BTree root) {
    SeqBTreeStackforPostT S;
    StackTNode stacktemp;
    
    initStackPost(&S);
    
    while (1) {
        while (root != NULL) {
            stacktemp.btreeNode = *root;
            stacktemp.flag = 0;
            pushPost(&S, &stacktemp);
            root = root->left;
        }
        
        if (isEmptyPost(&S) == TRUE) 
            return;
        else {
            popPost(&S, &stacktemp);
            if (stacktemp.flag == 0) {
                stacktemp.flag = 1;
                pushPost(&S, &stacktemp);
                root = stacktemp.btreeNode.right;
            }
            else {
                printf("%d ", stacktemp.btreeNode.data);
            }
        }
    }
}

// 层序遍历（使用队列）
void LevelorderTraverse(BTree root) {
    BTree queue[100];
    int front = 0, rear = 0;
    
    if (root == NULL) return;
    
    queue[rear++] = root;
    
    while (front != rear) {
        root = queue[front++];
        printf("%d ", root->data);
        
        if (root->left != NULL) 
            queue[rear++] = root->left;
        if (root->right != NULL) 
            queue[rear++] = root->right;
    }
}

// 计算二叉树高度
int TreeHeight(BTree root) {
    int leftHeight, rightHeight;
    
    if (root == NULL) 
        return 0;
    
    leftHeight = TreeHeight(root->left);
    rightHeight = TreeHeight(root->right);
    
    return (leftHeight > rightHeight ? leftHeight : rightHeight) + 1;
}

// 计算二叉树节点数
int TreeNodeCount(BTree root) {
    if (root == NULL) 
        return 0;
    
    return TreeNodeCount(root->left) + TreeNodeCount(root->right) + 1;
}

// 计算叶子节点数
int LeafNodeCount(BTree root) {
    if (root == NULL) 
        return 0;
    
    if (root->left == NULL && root->right == NULL) 
        return 1;
    
    return LeafNodeCount(root->left) + LeafNodeCount(root->right);
}

// 交换二叉树的左右子树
void SwapTree(BTree root) {
    BTree temp;
    
    if (root == NULL) 
        return;
    
    temp = root->left;
    root->left = root->right;
    root->right = temp;
    
    SwapTree(root->left);
    SwapTree(root->right);
}

// 复制二叉树
BTree CopyTree(BTree root) {
    BTree newTree;
    
    if (root == NULL) 
        return NULL;
    
    newTree = (BTree)malloc(sizeof(BinTNode));
    if (newTree == NULL) 
        return NULL;
    
    newTree->data = root->data;
    newTree->left = CopyTree(root->left);
    newTree->right = CopyTree(root->right);
    
    return newTree;
}

// 查找节点
BTree FindNode(BTree root, int value) {
    BTree found = NULL;
    
    if (root == NULL) 
        return NULL;
    
    if (root->data == value) 
        return root;
    
    found = FindNode(root->left, value);
    if (found != NULL) 
        return found;
    
    return FindNode(root->right, value);
}

// 销毁二叉树
void DestroyTree(BTree root) {
    if (root == NULL) 
        return;
    
    DestroyTree(root->left);
    DestroyTree(root->right);
    free(root);
}

// 测试程序
int main() {
    BTree root = NULL, copy = NULL, found = NULL;
    
    // 创建测试二叉树
    //       1
    //      / \
    //     2   3
    //    / \   \
    //   4   5   6
    
    root = createNode(1);
    root->left = createNode(2);
    root->right = createNode(3);
    root->left->left = createNode(4);
    root->left->right = createNode(5);
    root->right->right = createNode(6);
    
    printf("=== 二叉树操作测试 ===\n\n");
    
    printf("1. 先序遍历（递归）：");
    PreorderTraverse(root);
    printf("\n");
    
    printf("2. 中序遍历（递归）：");
    InorderTraverse(root);
    printf("\n");
    
    printf("3. 后序遍历（递归）：");
    PostorderTraverse(root);
    printf("\n");
    
    printf("4. 层序遍历：");
    LevelorderTraverse(root);
    printf("\n");
    
    printf("5. 先序遍历（非递归）：");
    PreorderTraverseNonRecursive(root);
    printf("\n");
    
    printf("6. 中序遍历（非递归）：");
    InorderTraverseNonRecursive(root);
    printf("\n");
    
    printf("7. 后序遍历（非递归）：");
    PostorderTraverseNonRecursive(root);
    printf("\n");
    
    printf("\n8. 二叉树高度：%d\n", TreeHeight(root));
    printf("9. 节点总数：%d\n", TreeNodeCount(root));
    printf("10. 叶子节点数：%d\n", LeafNodeCount(root));
    
    printf("\n11. 查找节点5：");
    found = FindNode(root, 5);
    if (found != NULL) 
        printf("找到，值为 %d\n", found->data);
    else 
        printf("未找到\n");
    
    printf("\n12. 复制二叉树：");
    copy = CopyTree(root);
    printf("复制完成\n");
    printf("   复制树的先序遍历：");
    PreorderTraverse(copy);
    printf("\n");
    
    printf("\n13. 交换左右子树后：");
    SwapTree(root);
    printf("\n");
    printf("   先序遍历：");
    PreorderTraverse(root);
    printf("\n");
    
    // 销毁树
    DestroyTree(root);
    DestroyTree(copy);
    
    return 0;
}