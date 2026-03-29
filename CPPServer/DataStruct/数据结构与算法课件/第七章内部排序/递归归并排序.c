#include <stdio.h>
#include <stdlib.h>

typedef int ELEMType;

// 记录结构
typedef struct {
    ELEMType *data;
    int currentNum;
} myRcd;

// 辅助函数声明
void merge(myRcd *myarr, myRcd *tmplist, int l, int m, int n);

void mSort(myRcd *myarr, myRcd *tmplist, int left, int right)  // 递归实现
{
    int center;
    
    if (left == right)
        tmplist->data[left] = myarr->data[left];
    else {
        center = (left + right) / 2;
        mSort(myarr, tmplist, left, center);  // 归并排序数组myarr的前半段
        mSort(myarr, tmplist, center + 1, right);  // 归并排序数组myarr的后半段
        merge(myarr, tmplist, left, center, right);
    }
}

int main()
{
    int arr[] = {64, 34, 25, 12, 22, 11, 90};
    int n = sizeof(arr) / sizeof(arr[0]);
    myRcd myarr, tmplist;
    int *temp = (int *)malloc(n * sizeof(int));
    
    myarr.data = arr;
    myarr.currentNum = n;
    tmplist.data = temp;
    tmplist.currentNum = n;
    
    printf("排序前：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    mSort(&myarr, &tmplist, 0, n - 1);
    
    printf("排序后：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    free(temp);
    return 0;
}
