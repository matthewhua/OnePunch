#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0

// 栈结构定义
typedef struct {
    char data[100];
    int top;
} SeqStack;

// 栈操作函数声明
void initStack(SeqStack *s);
int push(SeqStack *s, char value);
int pop(SeqStack *s, char *value);
int isEmpty(SeqStack *s);

int checkBalance(char *str)
{
    SeqStack mys;
    int isBalanced = 1, inputchar = 0, stackchar = 0;
    
    initStack(&mys);
    
    while (*str != '\0' && isBalanced) {
        switch (*str) {
            case '(':  case '[':  case '{':
                push(&mys, *str);
                break;
            case ')':
                if (isEmpty(&mys) == TRUE)
                    isBalanced = 0;
                else {
                    pop(&mys, &stackchar);
                    if (stackchar != '(') {
                        printf("不平衡的符号：%c 和 %c.\n", stackchar, *str);
                        isBalanced = 0;
                    }
                }
                break;
            case ']':
                if (isEmpty(&mys) == TRUE)
                    isBalanced = 0;
                else {
                    pop(&mys, &stackchar);
                    if (stackchar != '[') {
                        printf("不平衡的符号：%c 和 %c.\n", stackchar, *str);
                        isBalanced = 0;
                    }
                }
                break;
            case '}':
                if (isEmpty(&mys) == TRUE)
                    isBalanced = 0;
                else {
                    pop(&mys, &stackchar);
                    if (stackchar != '{') {
                        printf("不平衡的符号：%c 和 %c.\n", stackchar, *str);
                        isBalanced = 0;
                    }
                }
                break;
            default: 
                break;
        }
        str++;
    }
    
    if (!isEmpty(&mys)) 
        isBalanced = 0;
    
    if (isBalanced)
        printf("括号平衡。\n");
    else 
        printf("括号不平衡。\n");
    
    return isBalanced;
}

int main()
{
    char expr[100];
    printf("请输入表达式：");
    scanf("%s", expr);
    checkBalance(expr);
    return 0;
}
