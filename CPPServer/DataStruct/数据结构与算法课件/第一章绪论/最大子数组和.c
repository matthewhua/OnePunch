#include <stdio.h>

int MaxSeq(int A[], int n, int C[])
{
    int MaxSum = 0;
    int sum = 0, i, count = 0;
    C[0] = C[1] = 0;  // 子数组的起点为C[0]，终点为C[1]
    for (i = 0; i < n; i++) {
        sum += A[i];
        count++;  // 子数组长度
        if (sum > MaxSum) {
            MaxSum = sum;
            C[1] = i;
            C[0] = C[1] - count + 1;
        }
        if (sum < 0) {
            sum = 0;
            count = 0;
        }
        printf("i=%d A=%d  sum=%d Max=%d\n", i, A[i], sum, MaxSum);
    }
    if (MaxSum < 0) {
        C[0] = C[1] = 0;
        return 0;
    }
    return MaxSum;
}

int main()
{
    int A[] = {-2, 1, -3, 4, -1, 2, 1, -5, 4};
    int C[2];  // 从C[0]到C[1]
    int n = sizeof(A) / sizeof(int);
    int sum = MaxSeq(A, n, C);
    if (sum > 0) 
        printf("最大子数组和为: %d   从 A[%d] 到 A[%d]\n", sum, C[0], C[1]);
    else 
        printf("全部数据均为负数，最大子数组和为0\n");
    return 0;
}
