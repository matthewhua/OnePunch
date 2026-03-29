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
int innerpriv(char op);
int outpriv(char op);

int expression(char *input, char *result)
{
    SeqStack mys;
    int counter = 0, stackchar = 0;
    char *p = input;
    
    initStack(&mys);
    push(&mys, '#');  // 特殊栈底
    
    while (isEmpty(&mys) == FALSE && *p != '\0') {
        switch (*p) {
            case '+':  case '-':  case '*':  case '/':  case '%':  // 运算符
            case '(':  case ')':  case '#':
                pop(&mys, &stackchar);
                if (innerpriv(stackchar) < outpriv(*p)) {  // 栈外符号优先级高入栈
                    push(&mys, stackchar);
                    push(&mys, *p);
                    ++p;
                }
                else if (innerpriv(stackchar) > outpriv(*p)) {  // 栈内符号优先级高输出
                    result[counter++] = stackchar;
                }
                else {
                    if (stackchar == '(') 
                        ++p;
                }
                break;
            case '0':  case '1':  case '2':  case '3':  case '4':  // 操作数
            case '5':  case '6':  case '7':  case '8':  case '9':
                result[counter++] = *p++;
                while (*p >= '0' && *p <= '9')  // 若下一个字符是数字，则处理
                    result[counter++] = *p++;
                result[counter++] = ' ';
                break;
        }
    }
    
    while (isEmpty(&mys) == FALSE) {
        pop(&mys, &stackchar);
        if (stackchar != '#' && stackchar != '(')
            result[counter++] = stackchar;
    }
    
    result[counter] = '\0';
    return 0;
}

int main()
{
    char input[100], result[100];
    printf("请输入中缀表达式：");
    scanf("%s", input);
    expression(input, result);
    printf("后缀表达式：%s\n", result);
    return 0;
}
