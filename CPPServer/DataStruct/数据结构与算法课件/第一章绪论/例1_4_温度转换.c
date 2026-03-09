#include <stdio.h>

int main(int argc, char **argv)
{
    double Ctemp, Ftemp;  // 分别代表两种温标值
    const double fac = 1.8, inc = 32.0;
    printf("输入摄氏温标值：");
    scanf("%lf", &Ctemp);
    Ftemp = Ctemp * fac + inc;
    printf("摄氏 %3.1lf 度对应的华氏温标值是：%3.1lf\n", Ctemp, Ftemp);
    return 0;
}
