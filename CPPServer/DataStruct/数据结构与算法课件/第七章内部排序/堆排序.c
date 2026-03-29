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

void swap(ELEMType *a, ELEMType *b)
{
    ELEMType temp = *a;
    *a = *b;
    *b = temp;
}

void ShiftDown(myRcd *myarr, int i, int n)  // 堆的调整算法
{
    int child;
    
    for (; i <= ((n / 2) - 1); i = child) {
        child = i * 2 + 1;
        if ((child != (n - 1)) && (myarr->data[child + 1] > myarr->data[child]))
            child++;
        if (myarr->data[i] < myarr->data[child])
            swap(&myarr->data[i], &myarr->data[child]);
    }
}

void HeapSort(myRcd *myarr)  // 堆排序算法
{
    int i;
    
    for (i = (myarr->currentNum / 2 - 1); i >= 0; i--)
        ShiftDown(myarr, i, myarr->currentNum);
    
    for (i = myarr->currentNum - 1; i >= 1; i--) {
        swap(&myarr->data[0], &myarr->data[i]);
        ShiftDown(myarr, 0, i);
    }
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
    
    HeapSort(&myarr);
    
    printf("排序后：");
    for (int i = 0; i < n; i++)
        printf("%d ", arr[i]);
    printf("\n");
    
    return 0;
}
