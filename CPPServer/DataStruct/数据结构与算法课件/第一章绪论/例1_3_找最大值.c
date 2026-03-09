#include <stdio.h>

int largest(int *array, int n)  // 找最大值
{
    int currlarge = array[0];  // 保存目前得到的最大值
    int i;
    for (i = 1; i < n; i++)  // 对数组中的每个元素进行处理
        if (array[i] > currlarge)
            currlarge = array[i];  // 如果大于目前已经找到的，则更新最大值
    return currlarge;  // 返回最大值
}

int main()
{
    int arr[] = {3, 5, 2, 8, 1, 9, 4};
    int n = sizeof(arr) / sizeof(int);
    printf("数组中的最大值是: %d\n", largest(arr, n));
    return 0;
}
