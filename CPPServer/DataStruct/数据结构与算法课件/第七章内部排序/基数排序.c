#include <stdio.h>
#include <stdlib.h>

typedef int ELEMType;

// 记录结构
typedef struct {
    ELEMType *data;
    int currentNum;
} myRcd;

void RadixSort(myRcd *myarr, int k, int r, int count[])
// 对k位数、r进制的数据进行基数排序，count是辅助计数数组
{
    int i, j, rtok;
    myRcd mybrr;  // 辅助空间
    
    mybrr.data = (ELEMType *)malloc(myarr->currentNum * sizeof(ELEMType));
    
    for (i = 0, rtok = 1; i < k; i++, rtok *= r) {
        for (j = 0; j < r; j++) 
            count[j] = 0;  // 赋初值
        
        for (j = 0; j < myarr->currentNum; j++)
            count[myarr->data[j] / rtok % r]++;  // 计数
        
        for (j = 1; j < r; j++)
            count[j] = count[j - 1] + count[j];  // 计算下标范围
        
        for (j = myarr->currentNum - 1; j >= 0; j--)
            mybrr.data[--count[myarr->data[j] / rtok % r]] = myarr->data[j];
            // 根据count的值将数据放入辅助空间中
        
        for (j = 0; j < myarr->currentNum; j++)
            myarr->data[j] = mybrr.data[j];  // 将辅助空间中的数据复制回原数组中
    }
    
    free(mybrr.data);
}

int main()
{
    int arr[] = {170, 45, 75, 90, 802, 24, 2, 66};
    int n = sizeof(arr) / sizeof(arr[0]);
    myRcd myarr;
    int count[10];  // 十进制
    
    myarr.data = arr;
    myarr.currentNum = n;
    
    printf("排序前：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    RadixSort(&myarr, 3, 10, count);  // 3位数，十进制
    
    printf("排序后：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    return 0;
}
