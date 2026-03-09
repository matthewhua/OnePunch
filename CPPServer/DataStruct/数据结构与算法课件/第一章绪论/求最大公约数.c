#include <stdio.h>

int gcd(int data1, int data2)
{
    int m;
    if (data1 > data2) {
        m = data1;
        data1 = data2;
        data2 = m;
    }
    m = 0;
    while ((m = data1 % data2) != 0) {
        data1 = data2;
        data2 = m;
    }
    return data2;
}

int main(int argc, char **argv)
{
    int data1 = 0, data2 = 0;  // 读入两个正整数
    printf("请输入两个正整数，以空格分隔，回车结束 \n");
    scanf("%d %d", &data1, &data2);
    if (data1 < 0 || data2 < 0) 
        printf("不能输入负数\n");
    else
        printf("%d 和 %d 的最大公约数是 %d\n", data1, data2, gcd(data1, data2));
    return 0;
}
