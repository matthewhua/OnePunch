#include <stdio.h>
#include <stdlib.h>

#define TRUE 1
#define FALSE 0

// 栈结构定义
typedef struct {
    int data[100];
    int top;
} SeqStack;

// 栈操作函数声明
void initStack(SeqStack *s);
int push(SeqStack *s, int value);
int pop(SeqStack *s, int *value);
int isEmpty(SeqStack *s);

int calexpress(char *str)
{
    SeqStack operstack;
    int tempvalue, loper, roper, result;
    
    initStack(&operstack);
    
    while (*str != '\0') {
        if (*str >= '0' && *str <= '9') {  // 数字
            tempvalue = *str - '0';
            str++;
            while (*str >= '0' && *str <= '9')
                tempvalue = tempvalue * 10 + *str++ - '0';
            push(&operstack, tempvalue);  // 操作数入栈
        }
        else {
            switch (*str) {  // 运算符，进行计算
            case '+':
                pop(&operstack, &roper);  // 操作数2出栈
                pop(&operstack, &loper);  // 操作数1出栈
                result = loper + roper;   // 执行：操作数1+操作数2
                push(&operstack, result);
                break;
            case '-':
                pop(&operstack, &roper);
                pop(&operstack, &loper);
                result = loper - roper;
                push(&operstack, result);
                break;
            case '*':
                pop(&operstack, &roper);
                pop(&operstack, &loper);
                result = loper * roper;
                push(&operstack, result);
                break;
            case '/':
                pop(&operstack, &roper);
                pop(&operstack, &loper);
                result = loper / roper;
                push(&operstack, result);
                break;
            case '%':
                pop(&operstack, &roper);
                pop(&operstack, &loper);
                result = loper % roper;
                push(&operstack, result);
                break;
            }
        }
        str++;
    }
    
    if (isEmpty(&operstack) == FALSE) {  // 最后的结果
        pop(&operstack, &result);
    }
    
    return result;
}

int main()
{
    char expr[100];
    printf("请输入后缀表达式（如：23+45*-）：");
    scanf("%s", expr);
    printf("计算结果：%d\n", calexpress(expr));
    return 0;
}
