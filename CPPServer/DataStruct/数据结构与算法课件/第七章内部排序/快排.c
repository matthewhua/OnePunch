#include <stdio.h>
#include <stdlib.h>

typedef int ELEMType;

// 记录结构
typedef struct {
    ELEMType *data;
    int currentNum;
} myRcd;

// 辅助函数声明
void swap(ELEMType *a, ELEMType *b);
int partition(myRcd *myarr, int left, int right);

void QuickPass(myRcd *myarr, int left, int right)  // 递归实现
{
    int k;
    if (left < right) {
        k = partition(myarr, left, right);
        QuickPass(myarr, left, k - 1);  // 递归快速排序枢轴左侧的数据记录
        QuickPass(myarr, k + 1, right);  // 递归快速排序枢轴右侧的数据记录
    }
}

void QuickSort(myRcd *myarr)
{
    QuickPass(myarr, 0, myarr->currentNum - 1);
}

int main()
{
    int arr[] = {64, 34, 25, 12, 22, 11, 90};
    int n = sizeof(arr) / sizeof(arr[0]);
    myRcd myarr;
    myarr.data = arr;
    myarr.currentNum = n;
    
    printf("排序前：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    QuickSort(&myarr);
    
    printf("排序后：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    return 0;
}
