#include <stdio.h>

int fact(int n)
{
    int factv = 1, i;
    if (n < 0) return -1;
    if (n <= 1) return 1;
    for (i = 2; i <= n; i++) 
        factv *= i;
    return factv;
}

int main()
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
