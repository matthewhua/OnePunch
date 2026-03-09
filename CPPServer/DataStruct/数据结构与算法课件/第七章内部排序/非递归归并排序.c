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
int min(int a, int b);

int min(int a, int b)
{
    return (a < b) ? a : b;
}

void mSort(myRcd *myarr, myRcd *tmplist, int left, int right)
{
    int i, j;
    
    if (left == right) {
        tmplist->data[left] = myarr->data[left];
        return;
    }
    
    i = 1;
    while (i < myarr->currentNum) {
        for (j = 0; j < right; j += 2 * i) {
            merge(myarr, tmplist, j, j + i - 1, min(j + 2 * i - 1, right));
        }
        i = i * 2;
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
