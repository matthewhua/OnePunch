#include <stdio.h>

void fibonacci(int a[], int n)
{
    int i = 0;
    if (n <= 0) {
        printf("n值错误\n");
        return;
    }
    else if (n == 1) {
        a[0] = 1;
        return;
    }
    else {
        a[0] = a[1] = 1;
        for (i = 2; i < n; i++) {
            a[i] = a[i - 1] + a[i - 2];
        }
    }
}

int main()
{
    int n;
    printf("请输入要生成的斐波那契数列长度：");
    scanf("%d", &n);
    
    if (n <= 0) {
        printf("n值错误\n");
        return 1;
    }
    
    int fib[100];
    fibonacci(fib, n);
    
    printf("斐波那契数列前 %d 项为：\n", n);
    for (int i = 0; i < n; i++) {
        printf("%d ", fib[i]);
    }
    printf("\n");
    
    return 0;
}
