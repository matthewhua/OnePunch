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

int partition(myRcd *myarr, int mostleft, int mostright)  // 对调划分
{
    int left, right;
    ELEMType pivot;
    
    if (mostleft < mostright) {
        pivot = myarr->data[mostleft];  // 第一个元素选做枢轴
        swap(&myarr->data[mostleft], &myarr->data[mostright]);  // 枢轴放到最后的位置
        left = mostleft;  // left从左向右找
        right = mostright - 1;  // right从右向左找
        
        for (;;) {  // 循环查找左边大于枢轴、右边小于枢轴的元素
            while (myarr->data[left] <= pivot && left <= mostright - 1) {
                left++;
            }
            // 找到左边第一个大于枢轴的记录
            
            while (myarr->data[right] >= pivot && right >= (mostleft)) {
                right--;
            }
            // 找到右边第一个小于枢轴的记录
            
            if (left < right)  // 交换刚找到的两个元素
                swap(&myarr->data[left], &myarr->data[right]);
            else
                break;
        }
        
        swap(&myarr->data[mostright], &myarr->data[left]);  // 将枢轴放到正确的位置
    }
    
    return left;
}
