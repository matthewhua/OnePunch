#include <stdio.h>

int fact(int n)
{
    if (n == 0) 
        return 1;
    else 
        return n * fact(n - 1);
}

int main(int argc, char **argv)
{
    int n;
    printf("请输入一个正整数，回车结束\n");
    scanf("%d", &n);
    if (n < 0) 
        printf("不能输入负数\n");
    else 
        printf("%d 的阶乘是 %d\n", n, fact(n));
    return 0;
}
